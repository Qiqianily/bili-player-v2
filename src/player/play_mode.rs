#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    Normal,    // 正常播放
    Shuffle,   // 随机播放
    Repeat,    // 单曲循环
    RepeatAll, // 全曲循环
}
impl PlayMode {
    pub fn get_string(&self) -> String {
        match self {
            PlayMode::Normal => "顺序播放".to_string(),
            PlayMode::Shuffle => "随机播放".to_string(),
            PlayMode::Repeat => "单曲循环".to_string(),
            PlayMode::RepeatAll => "全曲循环".to_string(),
        }
    }
    pub fn from_string(s: &str) -> Option<PlayMode> {
        match s {
            "shuffle" => Some(PlayMode::Shuffle),
            "repeat" => Some(PlayMode::Repeat),
            "repeat_all" => Some(PlayMode::RepeatAll),
            _ => Some(PlayMode::Normal),
        }
    }
}
