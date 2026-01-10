use bili_player::{
    logger::init_logger,
    pb::{
        AddPlaylistRequest, AddPlaylistResponse, DeletedRequest, DeletedResponse, GetStateRequest,
        GetStateResponse, NextRequest, NextResponse, PauseRequest, PauseResponse, PlayBvidRequest,
        PlayBvidResponse, PlayRequest, PlayResponse, PreviousRequest, PreviousResponse,
        ResumeRequest, ResumeResponse, SeekRequest, SeekResponse, SetModelRequest,
        SetModelResponse, SetVolumeRequest, SetVolumeResponse, ShowPlayListRequest,
        ShowPlayListResponse, StopRequest, StopResponse,
        player_service_server::{PlayerService, PlayerServiceServer},
    },
    player::{
        audio_player::AudioPlayer, command::PlayerCommand, play_mode::PlayMode, state::PlayerState,
    },
};
use tokio::sync::{mpsc, oneshot};
use tonic::{Request, Response, Status, transport::Server};

/// 创建一个结构体，用来实现 rpc 中的 server
// #[derive(Default)]
pub struct PlayerServer {
    pub command_sender: mpsc::Sender<PlayerCommand>,
}
impl PlayerServer {
    pub fn new(command_sender: mpsc::Sender<PlayerCommand>) -> Self {
        Self { command_sender }
    }
}
/// 实现 PlayerService trait
#[tonic::async_trait]
impl PlayerService for PlayerServer {
    async fn play(&self, _request: Request<PlayRequest>) -> Result<Response<PlayResponse>, Status> {
        if (self.command_sender.send(PlayerCommand::Play).await).is_ok() {
            let result = PlayResponse {
                success: true,
                message: "音乐正在播放中".into(),
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("播放音乐时失败！"))
        }
    }

    async fn play_bvid(
        &self,
        request: Request<PlayBvidRequest>,
    ) -> Result<Response<PlayBvidResponse>, Status> {
        let input = request.into_inner();
        let info = format!("即将播放: {}", input.bvid);
        let _res = self
            .command_sender
            .send(PlayerCommand::PlayBvid(input))
            .await;
        let result = PlayBvidResponse {
            success: true,
            message: info,
        };
        Ok(Response::new(result))
    }

    async fn pause(
        &self,
        _request: Request<PauseRequest>,
    ) -> Result<Response<PauseResponse>, Status> {
        if (self.command_sender.send(PlayerCommand::Pause).await).is_ok() {
            let result = PauseResponse {
                success: true,
                message: "暂停播放".into(),
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("暂停播放时失败！"))
        }
    }

    async fn next(&self, _request: Request<NextRequest>) -> Result<Response<NextResponse>, Status> {
        if (self.command_sender.send(PlayerCommand::Next).await).is_ok() {
            let result = NextResponse {
                success: true,
                message: "成功切换到下一首歌曲".into(),
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("切换下一首歌曲时失败！"))
        }
    }

    async fn previous(
        &self,
        _request: Request<PreviousRequest>,
    ) -> Result<Response<PreviousResponse>, Status> {
        if (self.command_sender.send(PlayerCommand::Previous).await).is_ok() {
            let result = PreviousResponse {
                success: true,
                message: "成功切换到上一首歌曲".into(),
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("切换上一首歌曲时失败！"))
        }
    }

    async fn stop(&self, _request: Request<StopRequest>) -> Result<Response<StopResponse>, Status> {
        if (self.command_sender.send(PlayerCommand::Stop).await).is_ok() {
            let result = StopResponse {
                success: true,
                message: "停止播放".into(),
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("停止播放时失败！"))
        }
    }
    async fn resume(
        &self,
        _request: Request<ResumeRequest>,
    ) -> Result<Response<ResumeResponse>, Status> {
        if (self.command_sender.send(PlayerCommand::Resume).await).is_ok() {
            let result = ResumeResponse {
                success: true,
                message: "恢复播放".into(),
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("恢复播放时失败！"))
        }
    }
    async fn set_model(
        &self,
        request: Request<SetModelRequest>,
    ) -> Result<Response<SetModelResponse>, Status> {
        let input = request.into_inner();
        let model = PlayMode::from_string(input.model.as_str()).unwrap();
        if (self
            .command_sender
            .send(PlayerCommand::SetModel(input))
            .await)
            .is_ok()
        {
            let info = format!("{} 模式设置成功!", model.get_string());
            let result = SetModelResponse {
                success: true,
                message: info,
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("设置播放模式失败"))
        }
    }
    async fn add_playlist(
        &self,
        _request: Request<AddPlaylistRequest>,
    ) -> Result<Response<AddPlaylistResponse>, Status> {
        todo!()
    }
    async fn deleted(
        &self,
        _request: Request<DeletedRequest>,
    ) -> Result<Response<DeletedResponse>, Status> {
        todo!()
    }
    async fn get_state(
        &self,
        _request: Request<GetStateRequest>,
    ) -> Result<Response<GetStateResponse>, Status> {
        // 创建一个 oneshot channel
        let (sender, receiver) = oneshot::channel::<PlayerState>();
        // 发送消息到播放器
        if (self
            .command_sender
            .send(PlayerCommand::GetState(sender))
            .await)
            .is_ok()
        {
            // 等待响应
            match receiver.await {
                Ok(state) => {
                    let result = GetStateResponse {
                        success: true,
                        message: state.to_string(),
                    };
                    return Ok(Response::new(result));
                }
                Err(_) => Err(Status::internal("获取播放器状态失败")),
            }
        } else {
            Err(Status::internal("获取播放器状态失败"))
        }
    }
    async fn show_play_list(
        &self,
        _request: Request<ShowPlayListRequest>,
    ) -> Result<Response<ShowPlayListResponse>, Status> {
        todo!()
    }
    async fn set_volume(
        &self,
        request: Request<SetVolumeRequest>,
    ) -> Result<Response<SetVolumeResponse>, Status> {
        let input = request.into_inner();
        if (self
            .command_sender
            .send(PlayerCommand::SetVolume(input))
            .await)
            .is_ok()
        {
            let result = SetVolumeResponse {
                success: true,
                message: "音量设置成功".into(),
            };
            return Ok(Response::new(result));
        } else {
            Err(Status::internal("设置音量时失败！"))
        }
    }
    async fn seek(&self, _request: Request<SeekRequest>) -> Result<Response<SeekResponse>, Status> {
        todo!()
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    init_logger("info").await?;
    let file = "musics.txt";
    let (mut player, command_sender) = AudioPlayer::new(file).await?;
    // 启动播放服务
    tokio::task::spawn({
        async move {
            if let Err(e) = player.run().await {
                tracing::error!("Player error: {}", e);
            }
        }
    });
    // 如果要启动就播放，需要发送 PlayerCommand::Play 信号
    // command_sender.send(PlayerCommand::Play).await?;
    // grpc 服务地址
    let addr = "[::1]:50052".parse().unwrap();
    // 创建grpc服务
    let svc = PlayerServer::new(command_sender);
    tracing::info!("UserServiceServer listening on {addr}");
    // 启动服务
    Server::builder()
        .add_service(PlayerServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
