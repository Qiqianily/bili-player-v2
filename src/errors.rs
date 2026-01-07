#[derive(Debug, thiserror::Error)]
pub enum PlayerError {
    #[error("Response data parsing error: {0}")]
    RespDataParsingError(String),

    #[error("Fetch error: {0}")]
    FetchError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("GStreamer initialization failed: {0}")]
    GstInit(String),

    #[error("State transition failed: {0}")]
    StateTransition(String),

    #[error("Playlist error: {0}")]
    Playlist(String),

    #[error("Volume out of range: {0}")]
    VolumeRange(String),

    #[error("Audio element error: {0}")]
    AudioElement(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),
}

impl From<gstreamer::glib::BoolError> for PlayerError {
    fn from(err: gstreamer::glib::BoolError) -> Self {
        PlayerError::StateTransition(err.to_string())
    }
}

impl From<gstreamer::glib::Error> for PlayerError {
    fn from(err: gstreamer::glib::Error) -> Self {
        PlayerError::GstInit(err.to_string())
    }
}

pub type PlayerResult<T> = anyhow::Result<T, PlayerError>;
