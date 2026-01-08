use reqwest::Client;
use serde_json::Value;

use crate::errors::{PlayerError, PlayerResult};

const BASE_FETCH_AUDIO_API_URL: &str = "https://api.bilibili.com/x/player/playurl?fnval=16";
const BASE_FETCH_VIDEO_API_URL: &str = "https://api.bilibili.com/x/web-interface/view";
/// 获取音频URL
///
/// # 参数：
/// - `client`: 请求客户端
/// - `bvid`: 视频ID
/// - `cid`: 视频分P ID
/// # return
/// - `PlayerResult<String>`: 音频URL
/// # Examples
///
/// ```
/// use bili_player::fetch::network;
/// use reqwest::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new();
///     let bvid = "BV1r7411p7R4";
///     let cid = "321818216";
///     let audio_url = network::fetch_audio_url(&client, bvid, cid).await.unwrap();
///     println!("Audio URL: {}", audio_url);
/// }
/// ```
pub async fn fetch_audio_url(client: &Client, bvid: &str, cid: &str) -> PlayerResult<String> {
    let url = format!("{}&bvid={}&cid={}", BASE_FETCH_AUDIO_API_URL, bvid, cid);
    // tracing::info!("Fetching audio URL...");
    let response = client.get(&url).send().await?;
    let json: Value = response.json().await?;
    json["data"]["dash"]["audio"][0]["baseUrl"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| PlayerError::RespDataParsingError("解析音频URL失败".to_string()))
}
#[derive(serde::Deserialize, Debug)]
pub struct Owner {
    pub name: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct VideoData {
    pub bvid: String,
    pub title: String,
    pub cid: i64,
    pub owner: Owner,
}
#[derive(serde::Deserialize, Debug)]
struct ApiResponse<T> {
    data: T,
}
/// 请求视频信息，获取相关数据
///
/// # 参数
/// - `client`: 请求客户端
/// - `bvid`: 视频的BV号
/// # 返回值
/// - `PlayerResult<VideoData>`: 视频数据
///
/// # example
/// ```
/// async fn main() {
///     let client = Client::new();
///     let bvid = "BV1r7411p7R4";
///     let video_data = network::fetch_video_data(&client, bvid).await.unwrap();
///     println!("Video Title: {}", video_data.title);
/// }
/// ```
pub async fn fetch_video_data(client: &Client, bvid: &str) -> PlayerResult<VideoData> {
    let url = format!("{}?bvid={}", BASE_FETCH_VIDEO_API_URL, bvid);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| PlayerError::FetchError(format!("Fetch video data failed:{e}")))?;
    let mut api_response: ApiResponse<VideoData> = response
        .json()
        .await
        .map_err(|e| PlayerError::FetchError(format!("Fetch video data failed:{e}")))?;
    api_response.data.bvid = bvid.to_string();
    Ok(api_response.data)
}
