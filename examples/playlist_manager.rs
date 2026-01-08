use bili_player::player::{model::MusicInfo, play_mode::PlayMode, playlist::PlaylistManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = PlaylistManager::new();
    let music_info = MusicInfo {
        bvid: "BV1oZqqBZEGZ".to_string(),
        cid: "34856567673".to_string(),
        title: "西楼别序".to_string(),
        artist: Some("赵兮月".to_string()),
        duration: 227,
        owner: "夕照影音".to_string(),
    };
    let music_info2 = MusicInfo {
        bvid: "BV1rU4y1Y71M".to_string(),
        cid: "794672205".to_string(),
        title: "长大成人".to_string(),
        artist: Some("范茹".to_string()),
        duration: 217,
        owner: "OYMusicChannel".to_string(),
    };
    let music_info3 = MusicInfo {
        bvid: "BV1r7411p7R4".to_string(),
        cid: "321818216".to_string(),
        title: "青花瓷".to_string(),
        artist: Some("周杰伦".to_string()),
        duration: 243,
        owner: "音乐无限".to_string(),
    };
    let music_info4 = MusicInfo {
        bvid: "BV16K411d7PR".to_string(),
        cid: "898162929".to_string(),
        title: "漂洋过海来看你".to_string(),
        artist: Some("刘明湘".to_string()),
        duration: 272,
        owner: "大头音乐8090".to_string(),
    };

    {
        let current_index = manager.current_index.lock().await;
        println!("当前播放列表索引1：{:?}", current_index); // None
    }
    manager.add_music(music_info).await;
    {
        let current_index = manager.current_index.lock().await;
        println!("当前播放列表索引2：{:?}", current_index); // Some(0)
    }
    manager.add_music(music_info2).await;
    {
        let current_index = manager.current_index.lock().await;
        println!("当前播放列表索引3：{:?}", current_index); // Some(0)
    }
    manager.add_music(music_info3).await;
    manager.add_music(music_info4).await;
    {
        let shuffle_order = manager.shuffle_order.lock().await;
        println!("当前播放列表 shuffle_order：{:?}", shuffle_order); // Some([1, 0])
    }
    manager.set_play_mode(PlayMode::Shuffle).await;
    {
        let shuffle_order = manager.shuffle_order.lock().await;
        println!("当前播放列表 shuffle_order：{:?}", shuffle_order); // Some([1, 0])
    }
    // playlist
    {
        let playlist = manager.playlist.lock().await;
        // println!("当前播放列表：{:?}", playlist); // Some([1, 0])
        println!("=== Playlist ({} items) ===", playlist.len());
        for (i, music) in playlist.iter().enumerate() {
            println!("[{}] {}", i, music);
        }
        println!("============================");
        // = = = Playlist (4 items) = = =
        // [0] 《西楼别序》(06:38) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:夕照影音
        // [1] 《西楼》(05:18) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:影音
        // [2] 《青花瓷》(05:18) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 周杰伦 上传者:影音
        // [3] 《上海滩》(05:18) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 周润发 上传者:影音
        // ============================
    }
    // playmode
    {
        let play_mode = manager.get_play_mode().await;
        println!("当前播放模式：{}", play_mode.get_string());
    }
    let play = manager.get_current_music().await.unwrap();
    println!("当前播放的音乐信息：{}", play);
    let result = manager.move_to_previous().await?;
    if result {
        println!("成功切换到上一首音乐");
        let previous_play = manager.get_current_music().await.unwrap();
        println!("上一首音乐信息：{}", previous_play);
    } else {
        println!("切换到上一首音乐失败");
    }
    let result = manager.move_to_next().await?;
    if result {
        println!("成功切换到下一首音乐");
        let next_play = manager.get_current_music().await.unwrap();
        println!("下一首音乐信息1：{}", next_play);
    } else {
        println!("切换到下一首音乐失败");
    }
    let result = manager.move_to_next().await?;
    if result {
        println!("成功切换到下一首音乐");
        let next_play = manager.get_current_music().await.unwrap();
        println!("下一首音乐信息2：{}", next_play);
    } else {
        println!("切换到下一首音乐失败");
    }

    // delete index
    manager.remove_music(0).await?;
    {
        let shuffle_order = manager.shuffle_order.lock().await;
        println!("当前播放列表 shuffle_order：{:?}", shuffle_order); // Some([1, 0])
    }
    {
        let playlist = manager.playlist.lock().await;
        // println!("当前播放列表：{:?}", playlist); // Some([1, 0])
        println!("=== Playlist ({} items) ===", playlist.len());
        for (i, music) in playlist.iter().enumerate() {
            println!("[{}] {}", i, music);
        }
        println!("============================");
        // = = = Playlist (4 items) = = =
        // [0] 《西楼别序》(06:38) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:夕照影音
        // [1] 《西楼》(05:18) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:影音
        // [2] 《青花瓷》(05:18) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 周杰伦 上传者:影音
        // [3] 《上海滩》(05:18) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 周润发 上传者:影音
        // ============================
    }
    let play = manager.get_current_music().await.unwrap();
    println!("当前播放的音乐信息：{}", play);
    let index = manager.find_music_index("BV1oZqdBZEGZ").await.unwrap();
    println!("《上海滩》的索引：{}", index);
    Ok(())
}

// print result
/*
当前播放列表索引1：None
当前播放列表索引2：Some(0)
当前播放列表索引3：Some(0)
当前播放列表 shuffle_order：Some([2, 0, 3, 1])
当前播放列表 shuffle_order：Some([3, 1, 0, 2])
=== Playlist (4 items) ===
[0] 《西楼别序》(06:38) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:夕照影音
[1] 《西楼》(05:18) bvid: BV1oZccBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:影音
[2] 《青花瓷》(05:18) bvid: BV1oZhqBZEGZ cid: 34856567673 演唱: 周杰伦 上传者:影音
[3] 《上海滩》(05:18) bvid: BV1oZqdBZEGZ cid: 34856567673 演唱: 周润发 上传者:影音
============================
当前播放模式：随机播放
当前播放的音乐信息：《西楼别序》(06:38) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:夕照影音
成功切换到上一首音乐
上一首音乐信息：《西楼》(05:18) bvid: BV1oZccBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:影音
成功切换到下一首音乐
下一首音乐信息1：《西楼别序》(06:38) bvid: BV1oZqqBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:夕照影音
成功切换到下一首音乐
下一首音乐信息2：《青花瓷》(05:18) bvid: BV1oZhqBZEGZ cid: 34856567673 演唱: 周杰伦 上传者:影音
当前播放列表 shuffle_order：Some([2, 0, 1])
=== Playlist (3 items) ===
[0] 《西楼》(05:18) bvid: BV1oZccBZEGZ cid: 34856567673 演唱: 赵夕月 上传者:影音
[1] 《青花瓷》(05:18) bvid: BV1oZhqBZEGZ cid: 34856567673 演唱: 周杰伦 上传者:影音
[2] 《上海滩》(05:18) bvid: BV1oZqdBZEGZ cid: 34856567673 演唱: 周润发 上传者:影音
============================
当前播放的音乐信息：《青花瓷》(05:18) bvid: BV1oZhqBZEGZ cid: 34856567673 演唱: 周杰伦 上传者:影音
《上海滩》的索引：2
*/
