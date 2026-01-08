use gstreamer::{
    GenericFormattedValue,
    format::FormattedValue,
    prelude::{ElementExt, ElementExtManual, GstBinExt, GstBinExtManual},
};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

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
        }
    }
}
pub struct PlaybackManager {
    pub pipeline: gstreamer::Pipeline,             // 播放通道
    pub playback_state: Arc<Mutex<PlaybackState>>, // 播放状态
    pub current_music: Mutex<Option<MusicInfo>>,   // 当前播放音乐信息
    pub eos_sender: Option<mpsc::Sender<()>>,      // 播放结束信号发送器
}

impl PlaybackManager {
    /// PlaybackManager 的构造函数
    pub fn new(pipeline: gstreamer::Pipeline, eos_sender: Option<mpsc::Sender<()>>) -> Self {
        Self {
            pipeline,
            playback_state: Arc::new(Mutex::new(PlaybackState::Idle)),
            current_music: Mutex::new(None),
            eos_sender,
        }
    }
    /// 获取播放管道
    pub fn get_pipeline(&self) -> &gstreamer::Pipeline {
        &self.pipeline
    }
    /// 播放音频
    pub async fn play_music(
        &mut self,
        client: &reqwest::Client,
        music: &MusicInfo,
        volume: f64,
    ) -> PlayerResult<()> {
        // 请求音频的 url
        let url = fetch_and_verify_audio_url(client, music.bvid.as_str(), music.cid.as_str())
            .await
            .map_err(|_| PlayerError::FetchError("Fetch audio URL failed".into()))?;
        // 停止先前的播放
        self.stop().await?;
        // 构建播放管道
        self.build_pipeline(url.as_str(), volume).await?;
        // 开始播放
        self.pipeline
            .set_state(gstreamer::State::Playing)
            .map_err(|e| {
                PlayerError::StateTransition(format!("Failed to start playback: {}", e))
            })?;
        // 存储当前播放的 music
        {
            let mut current_music = self.current_music.lock().await;
            *current_music = Some(music.clone());
            // tracing::info!("set current music {}", music.title);
        }
        {
            let mut state = self.playback_state.lock().await;
            *state = PlaybackState::Playing;
            tracing::info!("set playback state: {}", state.get_string());
        }
        tracing::info!("Started playback: {}", music.title);
        // Watch GStreamer bus messages
        // self.watch_bus();
        // ✅ 获取总线
        let bus = self
            .pipeline
            .bus()
            .ok_or_else(|| PlayerError::Pipeline("Failed to get GStreamer bus".to_string()))?;
        // let state_arc = self.playback_state.clone();
        // let eos_sender = self.eos_sender.clone();
        for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
            match msg.view() {
                gstreamer::MessageView::Eos(_) => {
                    tracing::info!("{} 播放完成!", music.title);
                    if let Some(eos_sender_clone) = self.eos_sender.clone() {
                        let _ = eos_sender_clone.send(()).await;
                    }
                    break;
                }
                gstreamer::MessageView::Error(err) => {
                    tracing::error!(
                        "播放错误: {} (源: {})",
                        err.error(),
                        err.src().map(|s| s.to_string()).unwrap_or_default()
                    );
                }
                _ => {}
            }
        }
        // ✅ 启动后台任务处理消息
        // tokio::spawn(async move {
        //     loop {
        //         // 等待消息（最多 500ms）
        //         let msg = bus.timed_pop(gstreamer::ClockTime::from_mseconds(500));
        //         match msg {
        //             Some(msg) => {
        //                 use gstreamer::MessageView;
        //                 match msg.view() {
        //                     MessageView::Eos(..) => {
        //                         tracing::info!("EOS");
        //                         *state_arc.lock().await = PlaybackState::Ended;
        //                         if let Some(sender) = &eos_sender {
        //                             let res = sender.send(()).await;
        //                             if let Ok(res) = res {
        //                                 tracing::info!("EOS send result: {:?}", res);
        //                             }
        //                         }
        //                         break; // 播放结束，退出循环
        //                     }
        //                     MessageView::Error(err) => {
        //                         eprintln!("Error: {}", err.error());
        //                         *state_arc.lock().await = PlaybackState::Idle;
        //                         break;
        //                     }
        //                     // MessageView::StateChanged(sc) => {
        //                     //     if let Some(new_state) = match sc.current() {
        //                     //         gstreamer::State::Playing => Some(PlaybackState::Playing),
        //                     //         gstreamer::State::Paused => Some(PlaybackState::Paused),
        //                     //         gstreamer::State::Ready => Some(PlaybackState::Ready),
        //                     //         gstreamer::State::Null => Some(PlaybackState::Idle),
        //                     //         _ => None,
        //                     //     } {
        //                     //         *state_arc.lock().unwrap() = new_state;
        //                     //     }
        //                     // }
        //                     _ => {}
        //                 }
        //             }
        //             None => {
        //                 // 超时，继续循环（可加日志或退出条件）
        //             }
        //         }
        //     }
        // });
        // 清理
        self.pipeline.set_state(gstreamer::State::Null).unwrap();
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
    pub async fn stop(&self) -> PlayerResult<()> {
        // 如果是在播放状态
        {
            let mut state = self.playback_state.lock().await;
            if *state != PlaybackState::Idle {
                self.pipeline
                    .set_state(gstreamer::State::Null)
                    .map_err(|e| PlayerError::StateTransition(e.to_string()))?;
                *state = PlaybackState::Idle;
                tracing::info!("Playback paused");
            }
        }

        // 清理管道
        self.cleanup_pipeline().await?;
        Ok(())
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

    /// 清理播放器
    async fn cleanup_pipeline(&self) -> PlayerResult<()> {
        // 获取管道中的所有元素
        let children = self.pipeline.children();

        // 先停止所有元素
        for child in &children {
            child.set_state(gstreamer::State::Null).ok();
        }

        // 从管道中移除所有元素
        for child in children {
            self.pipeline
                .remove(&child)
                .map_err(|_| PlayerError::AudioElement("Failed to remove element".into()))?;
        }

        tracing::debug!("Pipeline cleaned up");
        Ok(())
    }

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
