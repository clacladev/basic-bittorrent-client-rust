use std::env;
use torrent::decode_torrent_file;
use tracker::tracker_get;

mod bencode;
mod torrent;
mod tracker;

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
        println!("Info Hash: {}", torrent.info.hash_hex()?);

        println!("Piece Length: {}", torrent.info.piece_length);
        println!("Piece Hashes:");
        torrent
            .info
            .pieces_hashes()?
            .iter()
            .for_each(|hash| println!("{}", hash));
    } else if command == "peers" {
        let file_path = &args[2];
        let torrent = decode_torrent_file(file_path)?;
        tracker_get(&torrent).await?;
        // println!("{}", tracker_response.);
    } else {
        println!("unknown command: {}", args[1])
    }

    Ok(())
}
