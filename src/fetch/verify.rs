use std::time::Duration;

use reqwest::{
    Client,
    header::{ACCEPT, RANGE, USER_AGENT},
};
use tokio::time::sleep;

use crate::{
    errors::{PlayerError, PlayerResult},
    fetch::network::fetch_audio_url,
};

/// 验证音频 URL 是否可用
///
/// # 参数
/// - `client`: 请求客户端
/// - `url`: 音频 URL
/// # 返回值
/// - `PlayerResult<bool>`: 验证结果
pub async fn verify_audio_url(client: &Client, url: &str) -> PlayerResult<bool> {
    let response = client
        .get(url)
        .header(USER_AGENT, "Mozilla/5.0 BiliDroid/..* (bbcallen@gmail.com)")
        .header(ACCEPT, "*/*")
        .header(RANGE, "bytes=0-1024")
        .header("Referer", "https://www.bilibili.com")
        .send()
        .await
        .map_err(|e| PlayerError::NetworkError(e.to_string()))?;

    Ok(response.status().is_success())
}

/// 请求并验证音频 URL 是否可用，如果不可用则重试 3 次
///
/// # 参数
/// - `client`: 请求客户端
/// - `bvid`: 视频 ID
/// - `cid`: 视频分 P ID
/// # 返回值
/// - `PlayerResult<String>`: 验证成功的音频 URL
/// # Examples
///
/// ```
/// use bili_player::fetch::verify::fetch_and_verify_audio_url;
/// use reqwest::Client;
///
/// #[tokio::main]
/// async fn main() {
///     let client = Client::new();
///     let bvid = "BV1r7411p7R4";
///     let cid = "321818216";
///
///     match fetch_and_verify_audio_url(&client, bvid, cid).await {
///         Ok(url) => println!("Audio URL verified: {}", url),
///         Err(e) => println!("Error verifying audio URL: {}", e),
///     }
/// }
/// ```
///
pub async fn fetch_and_verify_audio_url(
    client: &Client,
    bvid: &str,
    cid: &str,
) -> PlayerResult<String> {
    // 最大重试次数
    const MAX_RETRIES: u32 = 3;
    // 最初的重试延迟
    const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);
    // 最大重试延迟变量
    let mut retry_delay = INITIAL_RETRY_DELAY;

    for attempt in 1..=MAX_RETRIES {
        match fetch_audio_url(client, bvid, cid).await {
            Ok(url) => match verify_audio_url(client, &url).await {
                Ok(true) => return Ok(url),
                Ok(false) => {
                    tracing::info!("Verification failed for URL: {}", url);
                }
                Err(e) => {
                    tracing::error!("Error verifying URL: {}", e);
                }
            },
            Err(e) => {
                tracing::error!("Error fetching audio URL: {}", e);
            }
        }
        if attempt < MAX_RETRIES {
            tracing::info!("Retrying... Attempt {}/{}", attempt, MAX_RETRIES);
            sleep(retry_delay).await;
            // Exponential backoff
            retry_delay *= 2;
        }
    }

    Err(PlayerError::FetchError(
        "Max retries reached for fetching and verifying audio URL".to_string(),
    ))
}
