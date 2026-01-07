use bili_player::player::model::MusicInfo;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // init_logger("info").await?;
    let music_info = MusicInfo {
        bvid: "BV1oZqqBZEGZ".to_string(),
        cid: "34856567673".to_string(),
        title: "西楼别序".to_string(),
        artist: Some("赵夕月".to_string()),
        duration: 398,
        owner: "夕照影音".to_string(),
    };

    // tracing::info!("{}", music_info);
    println!("{}", music_info);

    Ok(())
}
