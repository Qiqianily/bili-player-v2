use gstreamer::format::FormattedValue;

use crate::player::{model::MusicInfo, play_mode::PlayMode, playback::PlaybackState};

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub playback_state: PlaybackState,
    pub current_position: Option<gstreamer::ClockTime>,
    pub duration: Option<gstreamer::ClockTime>,
    pub volume: u32,
    pub current_music: Option<MusicInfo>,
    pub play_mode: PlayMode,
    pub playlist_length: usize,
    pub current_index: Option<usize>,
}

impl std::fmt::Display for PlayerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let music_title = self
            .current_music
            .as_ref()
            .map(|music| music.title.clone())
            .unwrap_or("None".to_string());
        let music_info = self
            .current_music
            .as_ref()
            .map(|music| music.to_string())
            .unwrap_or("None".to_string());
        let current_index = match self.current_index {
            Some(idx) => idx + 1,
            None => 0,
        };
        write!(
            f,
            "{}:《{}》时长:{}/{}, 音量:{}, 播放模式:{}, 第{}个/共{}个。\n当前播放:{}",
            self.playback_state.show_info(),
            music_title,
            format_clock_time(self.current_position),
            format_clock_time(self.duration),
            self.volume,
            self.play_mode.get_string(),
            current_index,
            self.playlist_length,
            music_info
        )
    }
}

fn format_clock_time(time: Option<gstreamer::ClockTime>) -> String {
    match time {
        Some(t) if t.is_some() => {
            let secs = t.seconds(); // 安全：只有 is_some() 为 true 时才调用
            format!("{:02}:{:02}", secs / 60, secs % 60)
        }
        _ => "Unknown".to_string(),
    }
}
