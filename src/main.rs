use std::env;
use torrent::decode_torrent_file;

mod bencode;
mod torrent;

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = bencode::decode_bencoded_value(encoded_value)?;
        println!("{}", decoded_value);
    } else if command == "info" {
        let file_path = &args[2];
        let torrent = decode_torrent_file(file_path)?;
        println!("Tracker URL: {}", torrent.announce);
        println!("Length: {}", torrent.info.length);
        println!("Info Hash: {}", torrent.info.hash()?)
    } else {
        println!("unknown command: {}", args[1])
    }

    Ok(())
}
