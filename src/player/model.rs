#[derive(Clone, Debug)]
pub struct MusicInfo {
    pub bvid: String,           // BV1oZqqBZEGZ
    pub cid: String,            // 34856567673
    pub title: String,          // 西楼别序
    pub artist: Option<String>, // 赵夕月
    pub owner: String,          // 夕照影音
    pub duration: u64,          // 227 秒 03:47
}
/// 实现数据展示
impl std::fmt::Display for MusicInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let duration_str = format!("{:02}:{:02}", self.duration / 60, self.duration % 60);
        write!(
            f,
            "《{}》({}) bvid: {} cid: {} 演唱: {} 上传者:{}",
            self.title,
            duration_str,
            self.bvid,
            self.cid,
            self.artist.as_deref().unwrap_or("Unknown"),
            self.owner
        )
    }
}
