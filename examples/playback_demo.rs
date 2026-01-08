use bili_player::logger::init_logger;
use bili_player::player::model::MusicInfo;
use bili_player::player::playback::PlaybackManager;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger("debug").await?;
    // let music_info = MusicInfo {
    //     bvid: "BV1r7411p7R4".into(),
    //     cid: "321818216".into(),
    //     title: "青花瓷".into(),
    //     artist: Some("周杰伦".into()),
    //     owner: "音乐无限".into(),
    //     duration: 357,
    // };
    let music_info = MusicInfo {
        bvid: "BV1rU4y1Y71M".to_string(),
        cid: "794672205".to_string(),
        title: "长大成人".to_string(),
        artist: Some("范茹".to_string()),
        duration: 217,
        owner: "OYMusicChannel".to_string(),
    };

    gstreamer::init().unwrap();
    let (tx, mut rx) = mpsc::channel::<()>(1);
    let pipeline = gstreamer::Pipeline::new();

    let mut playback = PlaybackManager::new(pipeline, Some(tx));
    let client = reqwest::Client::new();
    // let volume_manager = VolumeManager::new(playback.pipeline.as_ref());
    // let volume = volume_manager.get_gstreamer_volume();
    playback.play_music(&client, &music_info, 0.10).await?;
    // 阻塞等待播放结束
    if let Some(msg) = rx.recv().await {
        println!("Received message: {:?}", msg);
    }
    Ok(())
}
