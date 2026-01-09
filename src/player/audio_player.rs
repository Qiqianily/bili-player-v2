use std::{future::pending, sync::Arc};

use tokio::{
    select,
    sync::{Mutex, mpsc},
};

use crate::{
    errors::{PlayerError, PlayerResult},
    player::{
        command::PlayerCommand, music_data::get_music_data, play_mode::PlayMode,
        playback::PlaybackManager, playlist::PlaylistManager, state::PlayerState,
        volume::VolumeManager,
    },
};

pub struct AudioPlayer {
    pub playback_manager: Arc<Mutex<PlaybackManager>>, // 播放管理
    pub volume_manager: Arc<VolumeManager>,            // 音量管理
    pub playlist_manager: Arc<PlaylistManager>,        // 播放列表管理
    pub client: Arc<reqwest::Client>,                  // HTTP客户端
    eos_receiver: Mutex<Option<mpsc::Receiver<()>>>,   // EOS事件接收器
    command_receiver: mpsc::Receiver<PlayerCommand>,   // 命令接收器
}
// pub state_sender: broadcast::Sender<PlayerState>, // 状态发送器
impl AudioPlayer {
    pub async fn new() -> PlayerResult<(Self, mpsc::Sender<PlayerCommand>)> {
        // 1. 初始化 GStreamer 和 Pipeline
        gstreamer::init().map_err(|e| PlayerError::GstInit(e.to_string()))?;
        let pipeline = gstreamer::Pipeline::new();

        // 2. 初始化播放列表
        let playlist_manager = Arc::new(PlaylistManager::new());
        let music_data = get_music_data();
        for music in music_data {
            playlist_manager.add_music(music).await;
        }

        // 3. 创建发送播放结束的信号通道
        let (eos_sender, eos_receiver) = mpsc::channel::<()>(1);
        // 4. 创建播放管理器
        let playback_manager = PlaybackManager::new(pipeline, Some(eos_sender));

        // 5. 创建 volume_manager 组件
        let volume_manager = Arc::new(VolumeManager::new());

        // 6. 创建发送播放结束的信号通道
        let (cmd_sender, cmd_receiver) = mpsc::channel::<PlayerCommand>(1);
        // 7. 创建播放器实例
        let player = Self {
            playback_manager: Arc::new(Mutex::new(playback_manager)),
            volume_manager,
            client: Arc::new(reqwest::Client::new()),
            playlist_manager,
            eos_receiver: Mutex::new(Some(eos_receiver)),
            command_receiver: cmd_receiver,
        };
        // // 启动后台任务
        // player.start_background_tasks(command_receiver).await?;
        Ok((player, cmd_sender))
    }
    /// 获取当前播放器状态
    pub async fn get_current_state(&self) -> PlayerState {
        let playback_manager = self.playback_manager.lock().await;
        let volume_manager = self.volume_manager.clone();
        let playlist_manager = self.playlist_manager.clone();
        PlayerState {
            playback_state: playback_manager.get_playback_state().await,
            current_position: playback_manager.get_current_position().await,
            duration: playback_manager.get_duration().await,
            volume: volume_manager.get_volume_percentage(),
            current_music: playlist_manager.get_current_music().await,
            play_mode: playlist_manager.get_play_mode().await,
            playlist_length: playlist_manager.get_playlist_len().await,
            current_index: playlist_manager.get_current_index().await,
        }
    }
    /// 运行播放器
    pub async fn run(&mut self) -> PlayerResult<()> {
        // 获取 EOS 接收器
        let mut eos_receiver = self.eos_receiver.lock().await.take();
        // 监听通道信号变化
        loop {
            select! {
                cmd = self.command_receiver.recv() => {
                    if let Some(command) = cmd {
                        self.handle_command(command).await?;
                    } else {
                        break; // sender dropped
                    }
                }
                _ = async {
                    if let Some(ref mut r) = eos_receiver {
                        r.recv().await
                    } else {
                        pending::<Option<()>>().await
                    }
                }, if eos_receiver.is_some() => {
                    println!("[EOS] Playing next track...");
                    let play_mode = self.playlist_manager.get_play_mode().await;
                    if play_mode == PlayMode::Repeat {
                        if let Some(music) = self.playlist_manager.get_current_music().await {
                            let client = self.client.clone();
                            let mut playback = self.playback_manager.lock().await;
                            let volume = self.volume_manager.get_gstreamer_volume();
                            playback.play_music(&client, &music, volume).await?;
                        }
                    } else {
                        // 触发下一首逻辑（内联，不走 command channel）
                        if self.playlist_manager.move_to_next().await?
                            && let Some(music) = self.playlist_manager.get_current_music().await {
                                let client = self.client.clone();
                                let mut playback = self.playback_manager.lock().await;
                                let volume = self.volume_manager.get_gstreamer_volume();
                                playback.play_music(&client, &music, volume).await?;
                            }
                    }
                }
            }
        }
        Ok(())
    }
    // 把命令处理逻辑抽到 handle_command
    async fn handle_command(&self, command: PlayerCommand) -> PlayerResult<()> {
        match command {
            PlayerCommand::Play => {
                if let Some(music) = self.playlist_manager.get_current_music().await {
                    let client = self.client.clone();
                    let mut playback = self.playback_manager.lock().await;
                    let volume = self.volume_manager.get_gstreamer_volume();
                    playback.play_music(&client, &music, volume).await?;
                }
            }
            PlayerCommand::PlayBvid(_req) => {
                // 1. 解析 BVID -> 获取音频 URL（调用 Bilibili API？）
                // 2. 加载到播放器
                // let uri = self.resolve_bvid_to_uri(&req.bvid).await?;
                // self.playback_manager.load_uri(&uri).await?;
                // self.playback_manager.play().await?;
            }
            PlayerCommand::Pause => {
                let playback = self.playback_manager.lock().await;
                playback.pause().await?;
            }
            PlayerCommand::Stop => {
                let mut playback = self.playback_manager.lock().await;
                playback.stop().await?;
            }
            PlayerCommand::Resume => {
                let playback = self.playback_manager.lock().await;
                playback.resume().await?;
            }
            PlayerCommand::Next => {
                if self.playlist_manager.move_to_next().await?
                    && let Some(music) = self.playlist_manager.get_current_music().await
                {
                    let client = self.client.clone();
                    let mut playback = self.playback_manager.lock().await;
                    let volume = self.volume_manager.get_gstreamer_volume();
                    playback.play_music(&client, &music, volume).await?;
                }
            }
            PlayerCommand::Previous => {
                if self.playlist_manager.move_to_previous().await?
                    && let Some(music) = self.playlist_manager.get_current_music().await
                {
                    let client = self.client.clone();
                    let mut playback = self.playback_manager.lock().await;
                    let volume = self.volume_manager.get_gstreamer_volume();
                    playback.play_music(&client, &music, volume).await?;
                }
            }
            PlayerCommand::SetModel(req) => {
                // 假设 SetModel 是切换播放模式（单曲、列表、随机等）
                let mode = PlayMode::from_string(req.model.as_str()).unwrap();
                self.playlist_manager.set_play_mode(mode).await;
            }
            PlayerCommand::SetVolume(_req) => {
                // self.volume_manager.set_volume(req.volume).await?;
            }
            PlayerCommand::AddPlaylist(_req) => {
                // for music in req.musics {
                //     self.playlist_manager.add_music(music).await;
                // }
            }
            PlayerCommand::Delete(_req) => {
                // self.playlist_manager.remove_by_id(&req.id).await;
            }
            PlayerCommand::GetState(sender) => {
                let state = self.get_current_state().await;
                let _ = sender.send(state); // 忽略发送失败（调用方可能已 drop）
            }
            PlayerCommand::ShowPlaylist() => {
                // 可能用于调试，或触发状态更新
                // self.log_playlist().await;
            }
            PlayerCommand::Seek(_position_micros) => {
                // let duration = std::time::Duration::from_micros(position_micros);
                // self.playback_manager.seek(duration).await?;
            }
        }
        Ok(())
    }
}
