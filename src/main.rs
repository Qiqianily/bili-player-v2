use bili_player::player::music_data::read_music_data;

fn main() {
    let file = "musics.txt";
    let results = read_music_data(file);
    for result in results {
        println!("{}", result);
    }
}
