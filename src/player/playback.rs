use gstreamer::{
    GenericFormattedValue,
    format::FormattedValue,
    prelude::{ElementExt, ElementExtManual},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::{
    sync::{Mutex, mpsc},
    task::JoinHandle,
};

use crate::{
    errors::{PlayerError, PlayerResult},
    fetch::verify::fetch_and_verify_audio_url,
    player::{audio_chain::AudioChainBuilder, model::MusicInfo},
};
/// 定义播放状态的枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,    // 初始待定状态
    Ready,   // 准备就绪状态
    Playing, // 正在播放状态
    Paused,  // 暂停播放状态
    Ended,   // 播放结束状态
    Error,   // 播放错误状态
    Stopped, // 停止播放状态
}
impl PlaybackState {
    pub fn get_string(&self) -> String {
        match self {
            Self::Idle => "初始待定状态".into(),
            Self::Ready => "准备就绪状态".into(),
            Self::Playing => "正在播放状态".into(),
            Self::Paused => "暂停播放状态".into(),
            Self::Ended => "播放结束状态".into(),
            Self::Error => "播放错误状态".into(),
            Self::Stopped => "停止播放状态".into(),
        }
    }
    pub fn show_info(&self) -> String {
        match self {
            Self::Idle => "初始待定".into(),
            Self::Ready => "准备就绪".into(),
            Self::Playing => "正在播放".into(),
            Self::Paused => "暂停播放".into(),
            Self::Ended => "播放结束".into(),
            Self::Error => "播放错误".into(),
            Self::Stopped => "停止播放".into(),
        }
    }
}
pub struct PlaybackManager {
    pub pipeline: gstreamer::Pipeline,             // 播放通道
    pub playback_state: Arc<Mutex<PlaybackState>>, // 播放状态
    pub current_music: Mutex<Option<MusicInfo>>,   // 当前播放音乐信息
    pub eos_sender: Option<mpsc::Sender<()>>,      // 播放结束信号发送器
    stop_flag: Arc<AtomicBool>,                    // 是否需要停止
    current_bus_watcher: Option<JoinHandle<()>>,   // 当前正在运行的后台监听任务句柄
}
// === 新增字段（用于管理后台监听任务）===
// stop_flag
// 标志位：通知 GStreamer 消息监听线程是否应主动退出
// 使用 `AtomicBool` 保证跨线程安全的无锁读写
// current_bus_watcher
// 保存当前正在运行的 `spawn_blocking` 任务的句柄
// 这样在切换歌曲或停止时，我们可以知道是否有旧任务需要清理
impl PlaybackManager {
    /// PlaybackManager 的构造函数
    pub fn new(pipeline: gstreamer::Pipeline, eos_sender: Option<mpsc::Sender<()>>) -> Self {
        Self {
            pipeline,
            playback_state: Arc::new(Mutex::new(PlaybackState::Idle)),
            current_music: Mutex::new(None),
            eos_sender,
            stop_flag: Arc::new(AtomicBool::new(false)),
            current_bus_watcher: None,
        }
    }
    /// 获取播放管道
    pub fn get_pipeline(&self) -> &gstreamer::Pipeline {
        &self.pipeline
    }
    /// 播放音乐
    pub async fn play_music(
        &mut self,
        client: &reqwest::Client,
        music: &MusicInfo,
        volume: f64,
    ) -> PlayerResult<()> {
        // 1️⃣ 获取音频真实播放 URL（调用 Bilibili API）
        let url = fetch_and_verify_audio_url(client, &music.bvid, &music.cid)
            .await
            .map_err(|_| PlayerError::FetchError("Fetch audio URL failed".into()))?;

        // 2️⃣ 停止当前正在播放的音乐（清理旧资源）
        //    这会触发 stop_flag 设置 + 旧任务清理 + pipeline 重置
        self.stop().await?;

        // 3️⃣ 为新歌曲构建 GStreamer 播放管道
        //    （内部会设置 URI、音量、总线等）
        self.build_pipeline(url.as_str(), volume).await?;

        // 4️⃣ 更新当前播放的音乐信息（供状态查询使用）
        {
            let mut current_music = self.current_music.lock().await;
            *current_music = Some(music.clone());
        }
        // 5️⃣ 更新全局播放状态为 "Playing"
        {
            let mut state = self.playback_state.lock().await;
            *state = PlaybackState::Playing;
            tracing::info!("Playback state set to: Playing");
        }

        // 6️⃣ 启动 GStreamer pipeline 开始播放
        self.pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| {
                PlayerError::StateTransition(format!("Failed to start playback: {}", e))
            })?;

        tracing::info!("Started playback: {}", music.title);

        // 7️⃣ 获取 GStreamer 消息总线（用于监听 EOS、Error 等事件）
        let bus = self
            .pipeline
            .bus()
            .ok_or_else(|| PlayerError::Pipeline("Failed to get GStreamer bus".to_string()))?;

        // 8️⃣ 为新监听任务创建独立的控制标志
        //    每次播放新歌都用新的 stop_flag，避免干扰
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone(); // 供后台任务使用

        // 9️⃣ 克隆需要在后台任务中使用的数据
        let eos_sender = self.eos_sender.clone(); // 通道可能为空（可选）
        let music_title = music.title.clone(); // 用于日志
        // let pipeline = self.pipeline.clone(); // 假设 pipeline 是 Arc<...>，否则需要调整

        // 🔟 启动后台线程监听 GStreamer 消息（关键！不阻塞 async 任务）
        let watcher_handle = tokio::task::spawn_blocking(move || {
            use gstreamer::MessageView;

            tracing::debug!("GStreamer bus watcher started for: {}", music_title);

            // 循环监听消息，直到收到 EOS、Error 或被要求停止
            loop {
                // ✅ 检查是否被外部请求停止（如切换歌曲、用户点击 Stop）
                if stop_flag_clone.load(Ordering::Relaxed) {
                    tracing::debug!(
                        "Bus watcher stopped by external request for: {}",
                        music_title
                    );
                    break;
                }

                // ⏳ 从总线获取消息（最多等待 1s，避免无限阻塞）
                match bus.timed_pop(gstreamer::ClockTime::from_seconds(1)) {
                    Some(msg) => match msg.view() {
                        // 🎯 播放正常结束（End Of Stream）
                        MessageView::Eos(_) => {
                            tracing::info!("Playback finished: {}", music_title);
                            // 通知主逻辑：可以播放下一首了
                            if let Some(sender) = &eos_sender {
                                let _ = sender.blocking_send(());
                            }
                            break; // 退出监听循环
                        }

                        // ❌ 播放发生错误
                        MessageView::Error(err) => {
                            tracing::error!(
                                "GStreamer playback error for {}: {}\nDebug: {}",
                                music_title,
                                err.error(),
                                err.debug().unwrap_or_default()
                            );
                            break; // 退出监听循环
                        }

                        // 其他消息（如缓冲、标签等）可选择忽略
                        _ => {}
                    },

                    // 超时（500ms 内无消息），继续循环
                    None => continue,
                }
            }

            tracing::debug!("GStreamer bus watcher exited for: {}", music_title);
        });

        // 🔚 保存新任务的控制信息，用于下次 stop() 时清理
        self.stop_flag = stop_flag;
        self.current_bus_watcher = Some(watcher_handle);

        // ✅ 立即返回！不等待播放结束
        //    此时歌曲已在后台播放，主逻辑可继续处理其他命令（如 Next、Stop）
        Ok(())
    }
    pub async fn play(&self) -> PlayerResult<()> {
        // let mut state = self.playback_state.lock().unwrap();
        // if *state == PlaybackState::Paused {
        //     self.pipeline
        //         .set_state(gstreamer::State::Playing)
        //         .map_err(|e| PlayerError::StateTransition(e.to_string()))?;
        //     *state = PlaybackState::Playing;
        //     tracing::info!("Started playback");
        // }

        Ok(())
    }
    /// 暂停播放
    pub async fn pause(&self) -> PlayerResult<()> {
        // 如果是在播放状态
        {
            let mut state = self.playback_state.lock().await;
            if *state == PlaybackState::Playing {
                self.pipeline
                    .set_state(gstreamer::State::Paused)
                    .map_err(|e| PlayerError::StateTransition(e.to_string()))?;
                *state = PlaybackState::Paused;
                tracing::info!("Playback paused");
            }
        }

        Ok(())
    }
    /// 恢复播放
    pub async fn resume(&self) -> PlayerResult<()> {
        // 如果是在暂停状态
        {
            let mut state = self.playback_state.lock().await;
            if *state == PlaybackState::Paused {
                self.pipeline
                    .set_state(gstreamer::State::Playing)
                    .map_err(|e| PlayerError::StateTransition(e.to_string()))?;
                *state = PlaybackState::Playing;
                tracing::info!("Playback resumed");
            }
        }
        Ok(())
    }
    /// 停止播放
    pub async fn stop(&mut self) -> PlayerResult<()> {
        // 1️⃣ 通知 GStreamer 消息监听线程：立即退出循环
        //    这样它就不会再尝试从已销毁的 bus 读取消息
        self.stop_flag.store(true, Ordering::Relaxed);
        // 2️⃣ 获取并移除当前的任务句柄（如果存在）
        if let Some(handle) = self.current_bus_watcher.take() {
            // 3️⃣ 启动一个后台 async 任务来等待 blocking 任务结束
            //    ⚠️ 不能直接 .await，因为 handle 是 spawn_blocking 任务（阻塞型），
            //    在 async 上下文中直接 await 会阻塞当前任务！
            tokio::spawn(async move {
                // 等待 spawn_blocking 任务完全退出
                // （正常情况下它会在下一次循环检查 stop_flag 后退出）
                let _ = handle.await;
                tracing::debug!("GStreamer bus watcher task exited cleanly");
            });
        }
        // 4️⃣ 停止 GStreamer pipeline（关键！释放音频设备、网络连接等资源）
        if self.pipeline.set_state(gstreamer::State::Null).is_err() {
            tracing::warn!("Failed to set GStreamer pipeline to Null state");
        }
        // 5️⃣ 清空当前播放的音乐信息
        {
            let mut current_music = self.current_music.lock().await;
            *current_music = None;
        }

        // 6️⃣ 更新全局播放状态为 "Stopped"
        {
            let mut state = self.playback_state.lock().await;
            *state = PlaybackState::Stopped;
            tracing::info!("Playback state set to: Stopped");
        }
        Ok(())
    }
    /// 获取当前播放状态
    pub async fn get_playback_state(&self) -> PlaybackState {
        let state = self.playback_state.lock().await;
        *state
    }
    /// 获取当前播放位置
    pub async fn get_current_position(&self) -> Option<gstreamer::ClockTime> {
        // 创建当前播放的位置查询对象
        let mut query = gstreamer::query::Position::new(gstreamer::Format::Time);
        if !self.pipeline.query(&mut query) {
            return None;
        }
        match query.result() {
            GenericFormattedValue::Time(Some(time)) if !time.is_none() => Some(time),
            _ => None,
        }
    }
    /// 获取音乐总时长
    pub async fn get_duration(&self) -> Option<gstreamer::ClockTime> {
        // 创建总时长查询对象
        let mut query = gstreamer::query::Duration::new(gstreamer::Format::Time);
        // 如果返回的是 false，说明查询失败
        if !self.pipeline.query(&mut query) {
            return None;
        }
        // 如果返回的是 None，说明没有设置总时长
        match query.result() {
            GenericFormattedValue::Time(Some(time)) if !time.is_none() => Some(time),
            _ => None,
        }
    }
    pub async fn seek(&self, position: gstreamer::ClockTime) -> PlayerResult<()> {
        {
            // 如果不是播放状态或暂停状态，直接返回错误
            let state = self.playback_state.lock().await;
            if *state == PlaybackState::Idle
                || *state == PlaybackState::Error
                || *state == PlaybackState::Ready
                || *state == PlaybackState::Ended
            {
                return Err(PlayerError::StateTransition(
                    "Cannot seek while not playing or paused".into(),
                ));
            }
        }

        let seek_flags = gstreamer::SeekFlags::FLUSH | gstreamer::SeekFlags::KEY_UNIT;

        if self.pipeline.seek_simple(seek_flags, position).is_err() {
            return Err(PlayerError::StateTransition("Seek failed".into()));
        }

        tracing::debug!("Sought to {:?}", position);
        Ok(())
    }
    // /// 添加 watch bus 用来接收播放状态变化
    // pub async fn watch_bus(&self) {
    //     let bus = self.pipeline.bus().expect("Pipeline should have a bus");

    //     // 克隆需要在回调中使用的字段
    //     let state_arc = self.playback_state.clone();
    //     let eos_sender_clone = self.eos_sender.clone(); // Option<Sender> 是 Clone 的
    // current_music 一般不需要在总线回调中修改，除非你要记录错误音乐等

    // let _ = bus.add_watch(async move |_, msg: &gstreamer::Message| {
    //     use gstreamer::MessageView;
    //     match msg.view() {
    //         MessageView::Eos(..) => {
    //             // 播放结束
    //             {
    //                 *state_arc.lock().await = PlaybackState::Ended;
    //             } // 🔒 锁释放
    //             if let Some(sender) = &eos_sender_clone {
    //                 let _ = sender.send(()); // 忽略发送失败（比如接收端已关闭）
    //             }
    //         }

    //         MessageView::Error(err) => {
    //             tracing::error!("GStreamer error: {}", err.error());
    //             if let Some(debug) = err.debug() {
    //                 // tracing::debug!("Debug info: {:?}", debug);
    //                 eprintln!("Debug info: {}", debug);
    //             }
    //             // 可选：更新状态为错误，或回到 Idle
    //             {
    //                 *state_arc.lock().unwrap() = PlaybackState::Idle;
    //             }
    //         }

    //         // MessageView::StateChanged(state_changed) => {
    //         //     // 注意：这个消息是元素状态变更，不是 pipeline 的最终状态
    //         //     // 通常我们关心的是 pipeline 的目标状态是否达成
    //         //     let new_state = state_changed.current();
    //         //     match new_state {
    //         //         gstreamer::State::Playing => {
    //         //             *state_arc.lock().unwrap() = PlaybackState::Playing;
    //         //         }
    //         //         gstreamer::State::Paused => {
    //         //             *state_arc.lock().unwrap() = PlaybackState::Paused;
    //         //         }
    //         //         gstreamer::State::Ready => {
    //         //             *state_arc.lock().unwrap() = PlaybackState::Ready;
    //         //         }
    //         //         gstreamer::State::Null => {
    //         //             *state_arc.lock().unwrap() = PlaybackState::Idle;
    //         //         }
    //         //         _ => return gstreamer::glib::ControlFlow::Continue, // 继续接收后续消息
    //         //     }
    //         // }

    //         // // 可选：处理缓冲、标签、时钟丢失等
    //         // MessageView::Buffering(buffering) => {
    //         //     // 比如网络流缓冲
    //         //     if buffering.percent() == 100 {
    //         //         // 缓冲完成，可以继续播放
    //         //     }
    //         // }
    //         _ => {
    //             // 其他消息可选择忽略
    //         }
    //     }
    //     gstreamer::glib::ControlFlow::Continue // 继续接收后续消息
    // });
    // }

    // /// 清理播放器
    // async fn cleanup_pipeline(&self) -> PlayerResult<()> {
    //     // 获取管道中的所有元素
    //     let children = self.pipeline.children();

    //     // 先停止所有元素
    //     for child in &children {
    //         child.set_state(gstreamer::State::Null).ok();
    //     }

    //     // 从管道中移除所有元素
    //     for child in children {
    //         self.pipeline
    //             .remove(&child)
    //             .map_err(|_| PlayerError::AudioElement("Failed to remove element".into()))?;
    //     }

    //     tracing::debug!("Pipeline cleaned up");
    //     Ok(())
    // }

    /// 构建播放器管道
    async fn build_pipeline(&mut self, url: &str, volume: f64) -> PlayerResult<()> {
        // 创建元素
        let (pipeline, _elements) = AudioChainBuilder::default()
            .with_url(url)
            .with_volume(volume)
            .build()
            .map_err(|e| {
                PlayerError::AudioElement(format!("Failed to build audio chain: {}", e))
            })?;
        pipeline
            .set_state(gstreamer::State::Ready)
            .map_err(|_| PlayerError::Pipeline("Failed to start playback".into()))?;
        self.pipeline = pipeline;
        Ok(())
    }
}
impl Drop for PlaybackManager {
    fn drop(&mut self) {
        // 确保资源被清理
        let _ = self.pipeline.set_state(gstreamer::State::Null);
    }
}
