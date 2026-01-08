use crate::player::{event::EosEvent, model::MusicInfo, play_mode::PlayMode};

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub is_playing: bool,
    pub current_position: Option<gstreamer::ClockTime>,
    pub duration: Option<gstreamer::ClockTime>,
    pub volume: u32,
    pub current_music: Option<MusicInfo>,
    pub play_mode: PlayMode,
    pub playlist_length: usize,
    pub current_index: Option<usize>,
    pub last_eos_event: Option<EosEvent>,
    pub error: Option<String>,
}

impl PlayerState {
    pub fn is_error(&self) -> bool {
        self.error.is_some()
            || self
                .last_eos_event
                .as_ref()
                .map(|e| e.is_error())
                .unwrap_or(false)
    }
}
