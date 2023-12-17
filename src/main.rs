use cli::Command;
use handshake::handshake;
use std::env;
use torrent::decode_torrent_file;
use tracker::tracker_get;

mod bencode;
mod cli;
mod handshake;
mod torrent;
mod tracker;

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command_str = &args[1];

    let command = Command::from_str(command_str);
    if let None = command {
        println!("Unknown command: {}", args[1]);
        return Ok(());
    }

    let command = command.unwrap();
    match command {
        Command::Decode => {
            let _ = execute_decode_command(&args[2]);
        }
        Command::Info => {
            let _ = execute_info_command(&args[2]);
        }
        Command::Peers => {
            let _ = execute_peers_command(&args[2]).await;
        }
        Command::Handshake => {
            let _ = execute_handshake_command(&args[2]).await;
        }
    }

    Ok(())
}

// ---
// Commands bodies

fn execute_decode_command(encoded_value: &str) -> anyhow::Result<()> {
    let decoded_value = bencode::decode_bencoded_value(encoded_value)?;
    println!("{}", decoded_value);
    Ok(())
}

fn execute_info_command(file_path: &str) -> anyhow::Result<()> {
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
    Ok(())
}

async fn execute_peers_command(file_path: &str) -> anyhow::Result<()> {
    let torrent = decode_torrent_file(file_path)?;
    let tracker_response = tracker_get(&torrent).await?;
    tracker_response
        .peers()
        .iter()
        .for_each(|peer| println!("{peer}"));
    Ok(())
}

async fn execute_handshake_command(file_path: &str) -> anyhow::Result<()> {
    let torrent = decode_torrent_file(file_path)?;
    let tracker_response = tracker_get(&torrent).await?;
    let peers = tracker_response.peers();
    let target_peer = peers.first().unwrap();
    let info_hash = torrent.info.hash_bytes()?;
    let peer_id = handshake(target_peer, &info_hash).await?;
    println!("Peer ID: {}", peer_id);
    Ok(())
}
