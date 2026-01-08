use crate::player::model::MusicInfo;
pub fn get_music_data() -> Vec<MusicInfo> {
    let music_info1 = MusicInfo {
        bvid: "BV1oZqqBZEGZ".to_string(),
        cid: "34856567673".to_string(),
        title: "西楼别序".to_string(),
        artist: Some("赵兮月".to_string()),
        duration: 227,
        owner: "夕照影音".to_string(),
    };
    let music_info2 = MusicInfo {
        bvid: "BV1rU4y1Y71M".to_string(),
        cid: "794672205".to_string(),
        title: "长大成人".to_string(),
        artist: Some("范茹".to_string()),
        duration: 217,
        owner: "OYMusicChannel".to_string(),
    };
    let music_info3 = MusicInfo {
        bvid: "BV1r7411p7R4".to_string(),
        cid: "321818216".to_string(),
        title: "青花瓷".to_string(),
        artist: Some("周杰伦".to_string()),
        duration: 243,
        owner: "音乐无限".to_string(),
    };
    let music_info4 = MusicInfo {
        bvid: "BV16K411d7PR".to_string(),
        cid: "898162929".to_string(),
        title: "漂洋过海来看你".to_string(),
        artist: Some("刘明湘".to_string()),
        duration: 272,
        owner: "大头音乐8090".to_string(),
    };
    vec![music_info1, music_info2, music_info3, music_info4]
}

// /// 播放音频
// pub async fn play_music(
//     &mut self,
//     client: &reqwest::Client,
//     music: &MusicInfo,
//     volume: f64,
// ) -> PlayerResult<()> {
//     // 请求音频的 url
//     let url = fetch_and_verify_audio_url(client, music.bvid.as_str(), music.cid.as_str())
//         .await
//         .map_err(|_| PlayerError::FetchError("Fetch audio URL failed".into()))?;
//     // 停止先前的播放
//     self.stop().await?;
//     // 构建播放管道
//     self.build_pipeline(url.as_str(), volume).await?;
//     // 开始播放
//     self.pipeline
//         .set_state(gstreamer::State::Playing)
//         .map_err(|e| {
//             PlayerError::StateTransition(format!("Failed to start playback: {}", e))
//         })?;
//     // 存储当前播放的 music
//     {
//         let mut current_music = self.current_music.lock().await;
//         *current_music = Some(music.clone());
//         // tracing::info!("set current music {}", music.title);
//     }
//     {
//         let mut state = self.playback_state.lock().await;
//         *state = PlaybackState::Playing;
//         tracing::info!("set playback state: {}", state.get_string());
//     }
//     tracing::info!("Started playback: {}", music.title);
//     // Watch GStreamer bus messages
//     // self.watch_bus();
//     // ✅ 获取总线
//     let bus = self
//         .pipeline
//         .bus()
//         .ok_or_else(|| PlayerError::Pipeline("Failed to get GStreamer bus".to_string()))?;
//     // let state_arc = self.playback_state.clone();
//     // let eos_sender = self.eos_sender.clone();
//     for msg in bus.iter_timed(gstreamer::ClockTime::NONE) {
//         match msg.view() {
//             gstreamer::MessageView::Eos(_) => {
//                 tracing::info!("{} 播放完成!", music.title);
//                 if let Some(eos_sender_clone) = self.eos_sender.clone() {
//                     let _ = eos_sender_clone.send(()).await;
//                 }
//                 break;
//             }
//             gstreamer::MessageView::Error(err) => {
//                 tracing::error!(
//                     "播放错误: {} (源: {})",
//                     err.error(),
//                     err.src().map(|s| s.to_string()).unwrap_or_default()
//                 );
//             }
//             _ => {}
//         }
//     }
//     // ✅ 启动后台任务处理消息
//     // tokio::spawn(async move {
//     //     loop {
//     //         // 等待消息（最多 500ms）
//     //         let msg = bus.timed_pop(gstreamer::ClockTime::from_mseconds(500));
//     //         match msg {
//     //             Some(msg) => {
//     //                 use gstreamer::MessageView;
//     //                 match msg.view() {
//     //                     MessageView::Eos(..) => {
//     //                         tracing::info!("EOS");
//     //                         *state_arc.lock().await = PlaybackState::Ended;
//     //                         if let Some(sender) = &eos_sender {
//     //                             let res = sender.send(()).await;
//     //                             if let Ok(res) = res {
//     //                                 tracing::info!("EOS send result: {:?}", res);
//     //                             }
//     //                         }
//     //                         break; // 播放结束，退出循环
//     //                     }
//     //                     MessageView::Error(err) => {
//     //                         eprintln!("Error: {}", err.error());
//     //                         *state_arc.lock().await = PlaybackState::Idle;
//     //                         break;
//     //                     }
//     //                     // MessageView::StateChanged(sc) => {
//     //                     //     if let Some(new_state) = match sc.current() {
//     //                     //         gstreamer::State::Playing => Some(PlaybackState::Playing),
//     //                     //         gstreamer::State::Paused => Some(PlaybackState::Paused),
//     //                     //         gstreamer::State::Ready => Some(PlaybackState::Ready),
//     //                     //         gstreamer::State::Null => Some(PlaybackState::Idle),
//     //                     //         _ => None,
//     //                     //     } {
//     //                     //         *state_arc.lock().unwrap() = new_state;
//     //                     //     }
//     //                     // }
//     //                     _ => {}
//     //                 }
//     //             }
//     //             None => {
//     //                 // 超时，继续循环（可加日志或退出条件）
//     //             }
//     //         }
//     //     }
//     // });
//     // 清理
//     self.pipeline.set_state(gstreamer::State::Null).unwrap();
//     Ok(())
// }
