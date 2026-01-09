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
            .unwrap_or_else(|| "None".to_string());
        let music_bvid = self
            .current_music
            .as_ref()
            .map(|music| music.bvid.clone())
            .unwrap_or_else(|| "None".to_string());
        let music_cid = self
            .current_music
            .as_ref()
            .map(|music| music.cid.clone())
            .unwrap_or_else(|| "None".to_string());
        let music_artist = self
            .current_music
            .as_ref()
            .and_then(|music| music.artist.clone()) // 如果 artist 是 Some，克隆出 String；否则 None
            .unwrap_or_else(|| "Unknown".to_string());
        let music_upper = self
            .current_music
            .as_ref()
            .map(|music| music.owner.clone())
            .unwrap_or_else(|| "None".to_string());
        let current_index = match self.current_index {
            Some(idx) => idx + 1,
            None => 0,
        };
        let duration = format_clock_time(self.duration);
        let music_info = format!(
            "《{}》({}) bvid: {} cid: {} 演唱: {} 上传者: {}",
            music_title, duration, music_bvid, music_cid, music_artist, music_upper
        );
        let music_info_show = format!(
            "《{}》-{}-{}-{}-{}-{}",
            music_title, duration, music_bvid, music_cid, music_artist, music_upper
        );
        write!(
            f,
            "{}:《{}》时长:{}/{}, 音量:{}, 播放模式:{}, 第{}个/共{}个。\n当前播放:{}\n保存：{}",
            self.playback_state.show_info(),
            music_title,
            format_clock_time(self.current_position),
            duration,
            self.volume,
            self.play_mode.get_string(),
            current_index,
            self.playlist_length,
            music_info,
            music_info_show
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
