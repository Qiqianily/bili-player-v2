use bili_player::fetch::{network::fetch_video_data, verify::fetch_and_verify_audio_url};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 client
    let client = reqwest::Client::new();
    // 获取视频数据（周杰伦的《青花瓷》）
    let video_data = fetch_video_data(&client, "BV16K411d7PR").await?;
    println!("Title: {:?}", video_data);
    // 获取音频 URL
    let audio_url =
        fetch_and_verify_audio_url(&client, &video_data.bvid, &video_data.cid.to_string()).await?;
    println!("Audio URL: {}", audio_url);
    Ok(())
}
