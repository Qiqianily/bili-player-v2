use crate::{
    pb::{
        AddPlaylistRequest, DeletedRequest, PlayBvidRequest, SetModelRequest, SetVolumeRequest,
        ShowMusicPageInfoResponse,
    },
    player::state::PlayerState,
};

#[derive(Debug)]
pub enum PlayerCommand {
    Play,
    PlayBvid(PlayBvidRequest),
    Pause,
    Next,
    Previous,
    Stop,
    Resume,
    SetModel(SetModelRequest),
    SetVolume(SetVolumeRequest),
    AddPlaylist(AddPlaylistRequest),
    Delete(DeletedRequest),
    GetState(tokio::sync::oneshot::Sender<PlayerState>),
    ShowMusicPageInfo {
        page: u32,
        sender: tokio::sync::oneshot::Sender<ShowMusicPageInfoResponse>,
    },
    Seek(u64),
}
