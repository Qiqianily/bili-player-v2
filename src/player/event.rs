#[derive(Debug, Clone, PartialEq)]
pub enum EosEvent {
    /// 正常播放结束
    NormalEnd,
    /// 播放错误结束
    ErrorEnd { error: String, should_retry: bool },
    /// 播放被中断（如手动停止）
    Interrupted,
    /// 网络问题导致的结束
    NetworkError {
        error: String,
        bvid: String,
        cid: String,
    },
    /// 播放列表结束
    PlaylistEnd,
    /// 跳过当前歌曲
    Skipped,
}

impl EosEvent {
    pub fn is_error(&self) -> bool {
        matches!(self, Self::ErrorEnd { .. } | Self::NetworkError { .. })
    }

    pub fn should_retry(&self) -> bool {
        match self {
            Self::ErrorEnd { should_retry, .. } => *should_retry,
            Self::NetworkError { .. } => true, // 网络错误通常可以重试
            _ => false,
        }
    }

    pub fn to_error_message(&self) -> Option<String> {
        match self {
            Self::ErrorEnd { error, .. } => Some(error.clone()),
            Self::NetworkError { error, .. } => Some(error.clone()),
            _ => None,
        }
    }
}

// 方便地从 GStreamer 错误转换
impl From<gstreamer::glib::Error> for EosEvent {
    fn from(err: gstreamer::glib::Error) -> Self {
        let error_str = err.to_string();
        let should_retry = !error_str.contains("fatal") && !error_str.contains("unsupported");
        EosEvent::ErrorEnd {
            error: error_str,
            should_retry,
        }
    }
}

// 方便地从网络错误转换
impl From<reqwest::Error> for EosEvent {
    fn from(err: reqwest::Error) -> Self {
        let bvid = String::new(); // 实际使用时需要传入
        let cid = String::new();

        let error_str = err.to_string();
        let is_timeout = error_str.contains("timeout") || error_str.contains("timed out");

        if is_timeout {
            EosEvent::NetworkError {
                error: format!("Network timeout: {}", error_str),
                bvid,
                cid,
            }
        } else {
            EosEvent::NetworkError {
                error: format!("Network error: {}", error_str),
                bvid,
                cid,
            }
        }
    }
}
