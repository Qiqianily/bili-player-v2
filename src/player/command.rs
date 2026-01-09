use crate::{
    pb::{AddPlaylistRequest, DeletedRequest, PlayBvidRequest, SetModelRequest, SetVolumeRequest},
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
    ShowPlaylist(),
    Seek(u64),
}
