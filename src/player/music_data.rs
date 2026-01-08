use crate::player::model::MusicInfo;
pub fn get_music_data() -> Vec<MusicInfo> {
    let music_info1 = MusicInfo {
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
    vec![music_info1, music_info2, music_info3, music_info4]
}
