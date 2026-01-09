use std::collections::VecDeque;

use rand::seq::SliceRandom;
use tokio::sync::{Mutex, RwLock};

use crate::{
    errors::{PlayerError, PlayerResult},
    fetch::network::fetch_video_data,
    player::{model::MusicInfo, play_mode::PlayMode},
};

pub struct PlaylistManager {
    pub playlist: Mutex<VecDeque<MusicInfo>>,     // 播放列表
    pub current_index: Mutex<Option<usize>>,      // 当前播放索引
    pub play_mode: RwLock<PlayMode>,              // 播放模式
    pub shuffle_order: Mutex<Option<Vec<usize>>>, // 随机播放顺序
}
impl Default for PlaylistManager {
    fn default() -> Self {
        Self::new()
    }
}
impl PlaylistManager {
    /// PlaylistManager 构造函数
    pub fn new() -> Self {
        Self {
            playlist: Mutex::new(VecDeque::new()),
            current_index: Mutex::new(None),
            play_mode: RwLock::new(PlayMode::Normal),
            shuffle_order: Mutex::new(None),
        }
    }
    /// 检查音乐是否在播放列表中
    pub async fn is_in_playlist(&self, bvid: &str) -> bool {
        self.playlist
            .lock()
            .await
            .iter()
            .any(|music| music.bvid == bvid)
    }
    /// 获取音乐信息
    pub async fn fetch_music_info(&self, bvid: &str) -> PlayerResult<MusicInfo> {
        // 实现获取音乐信息的逻辑
        let client = reqwest::Client::new();
        let video_data = fetch_video_data(&client, bvid).await?;
        let music_info = MusicInfo {
            bvid: video_data.bvid,
            cid: video_data.cid.to_string(),
            title: video_data.title,
            artist: None,
            owner: video_data.owner.name,
            duration: 0,
        };
        Ok(music_info)
    }
    /// 获取播放列表长度
    pub async fn get_playlist_len(&self) -> usize {
        self.playlist.lock().await.len()
    }
    /// 获取音乐索引
    pub async fn get_music_index(&self, bvid: &str) -> Option<usize> {
        self.playlist
            .lock()
            .await
            .iter()
            .position(|music| music.bvid == bvid)
    }
    /// 获取当前音乐索引
    pub async fn get_current_index(&self) -> Option<usize> {
        *self.current_index.lock().await
    }
    pub async fn add_will_play_music_into_playlist(&self, bvid: &str) -> PlayerResult<()> {
        let music_info = self.fetch_music_info(bvid).await?;
        {
            let mut playlist = self.playlist.lock().await;
            playlist.push_back(music_info);
            playlist.len()
        }; // 🔓 playlist 锁在这里释放
        // 获取这个音乐在列表中的索引
        let music_index = self.get_music_index(bvid).await.unwrap_or(0);
        // 如果当前没有选中的音乐，选择第一个
        {
            let mut current_index = self.current_index.lock().await;
            *current_index = Some(music_index);
        } // 🔓 current_index 锁释放

        // 重置随机播放顺序
        self.update_shuffle_order().await;
        Ok(())
    }
    /// 添加音乐到播放列表
    pub async fn add_music(&self, music: MusicInfo) {
        let new_len = {
            let mut playlist = self.playlist.lock().await;
            playlist.push_back(music);
            playlist.len()
        }; // 🔓 playlist 锁在这里释放

        // 如果当前没有选中的音乐，选择第一个
        {
            let mut current_index = self.current_index.lock().await;
            if current_index.is_none() && new_len > 0 {
                *current_index = Some(0);
            }
        } // 🔓 current_index 锁释放

        // 重置随机播放顺序
        self.update_shuffle_order().await;
    }
    /// 从播放列表中移除音乐
    pub async fn remove_music(&self, index: usize) -> PlayerResult<()> {
        // 判断是否越界
        {
            let mut playlist = self.playlist.lock().await;
            if index >= playlist.len() {
                return Err(PlayerError::Playlist("Index out of bounds".into()));
            }
            playlist.remove(index);
        }

        // 更新当前索引
        self.adjust_current_index_after_removal(index).await;

        // 重置随机播放顺序
        self.update_shuffle_order().await;

        Ok(())
    }
    /// 获取当前播放的音乐信息
    pub async fn get_current_music(&self) -> Option<MusicInfo> {
        let current_index = self.current_index.lock().await;
        let playlist = self.playlist.lock().await;
        // 只有当 current_index 是 Some(idx) 时，才会执行闭包，否则返回 None
        current_index.and_then(|idx| playlist.get(idx).cloned())
    }
    /// 下一首
    pub async fn move_to_next(&self) -> PlayerResult<bool> {
        // 获取当前播放模式
        let play_mode = self.get_play_mode().await;
        // 获取当前播放索引
        let mut current_index = self.current_index.lock().await;
        // 获取当前播放列表
        let playlist = self.playlist.lock().await;
        // 如果列表为空，返回 false
        if playlist.is_empty() {
            return Ok(false);
        }

        match play_mode {
            // // 如果是单曲播放模式
            // PlayMode::Repeat => Ok(true),
            // 如果是随机播放模/式，随机选择一首
            PlayMode::Shuffle => {
                let shuffle_order = self.shuffle_order.lock().await;
                if let Some(order) = shuffle_order.as_ref()
                    && let Some(current_idx) = *current_index
                    && let Some(pos) = order.iter().position(|&i| i == current_idx)
                {
                    let next_pos = (pos + 1) % order.len();
                    *current_index = Some(order[next_pos]);
                    return Ok(true);
                }

                Ok(false)
            }
            // 其他就直接下一首
            _ => {
                if let Some(idx) = *current_index {
                    if idx + 1 < playlist.len() {
                        *current_index = Some(idx + 1);
                        Ok(true)
                    } else if idx == playlist.len() - 1 {
                        *current_index = Some(0);
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    *current_index = Some(0);
                    Ok(true)
                }
            }
        }
    }
    /// 上一首
    pub async fn move_to_previous(&self) -> PlayerResult<bool> {
        let play_mode = self.get_play_mode().await;
        let mut current_index = self.current_index.lock().await;
        let playlist = self.playlist.lock().await;

        if playlist.is_empty() {
            return Ok(false);
        }

        match play_mode {
            PlayMode::Shuffle => {
                let shuffle_order = self.shuffle_order.lock().await;
                if let Some(order) = shuffle_order.as_ref()
                    && let Some(current_idx) = *current_index
                    && let Some(pos) = order.iter().position(|&i| i == current_idx)
                {
                    let prev_pos = if pos == 0 { order.len() - 1 } else { pos - 1 };
                    *current_index = Some(order[prev_pos]);
                    return Ok(true);
                }

                Ok(false)
            }
            _ => {
                if let Some(idx) = *current_index {
                    if idx > 0 {
                        *current_index = Some(idx - 1);
                        Ok(true)
                    } else if idx == 0 {
                        *current_index = Some(playlist.len() - 1);
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    *current_index = Some(0);
                    Ok(true)
                }
            }
        }
    }
    /// 设置播放模式
    pub async fn set_play_mode(&self, mode: PlayMode) {
        *self.play_mode.write().await = mode;
        // 如果是随机就重排
        if mode == PlayMode::Shuffle {
            self.update_shuffle_order().await;
        }
    }
    /// 获取 bvid 对应音乐的索引
    pub async fn find_music_index(&self, bvid: &str) -> Option<usize> {
        let playlist = self.playlist.lock().await;
        playlist.iter().position(|music| music.bvid == bvid)
    }
    /// 获取播放模式
    pub async fn get_play_mode(&self) -> PlayMode {
        *self.play_mode.read().await
    }
    /// 更新播放顺序
    async fn update_shuffle_order(&self) {
        let len = {
            let playlist = self.playlist.lock().await;
            playlist.len()
        }; // 🔓 playlist 锁释放
        let mut shuffle_order = self.shuffle_order.lock().await;
        if len == 0 {
            *shuffle_order = None;
            return; // 🔓 shuffle_order 锁释放
        }
        let mut order: Vec<usize> = (0..len).collect();
        order.shuffle(&mut rand::rng());
        *shuffle_order = Some(order);
    }
    /// 调整当前索引以适应删除操作
    async fn adjust_current_index_after_removal(&self, removed_index: usize) {
        // 获取 playlist 和 current index
        let playlist = self.playlist.lock().await;
        let mut current_index = self.current_index.lock().await;
        // 更新当前索引
        match *current_index {
            Some(idx) if idx == removed_index => {
                // 当前播放的歌曲被删除
                if playlist.is_empty() {
                    *current_index = None;
                } else if idx >= playlist.len() {
                    *current_index = Some(playlist.len() - 1);
                }
            }
            Some(idx) if idx > removed_index => {
                // 当前播放的歌曲在删除的歌曲之后
                *current_index = Some(idx - 1);
            }
            _ => {}
        }
    }
}
