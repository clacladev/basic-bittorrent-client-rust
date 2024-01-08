use cli::Command;
use std::env::{self};

use crate::torrent_client::TorrentClient;

mod bencode;
mod cli;
mod torrent_client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let command_str = &args[1];

    let command = Command::from_str(command_str);
    let Some(command) = command else {
        println!("Unknown command: {}", args[1]);
        return Ok(());
    };

    match command {
        Command::Decode => {
            execute_command_decode(&args[2])?;
        }
        Command::Info => {
            execute_command_info(&args[2])?;
        }
        Command::Peers => {
            execute_command_peers(&args[2]).await?;
        }
        Command::Handshake => {
            execute_command_handshake(&args[2]).await?;
        }
        Command::DownloadPiece => {
            let input_file_path = &args[4];
            let output_file_path = &args[3];
            let piece_index: &u32 = &args[5].parse()?;
            execute_command_download_piece(&input_file_path, &output_file_path, *piece_index)
                .await?;
        }
        Command::Download => {
            let input_file_path = &args[4];
            let output_file_path = &args[3];
            execute_command_download(&input_file_path, &output_file_path).await?;
        }
    }

    Ok(())
}

// ---
// Commands bodies

fn execute_command_decode(encoded_value: &str) -> anyhow::Result<()> {
    let decoded_value = bencode::decode_bencoded_value(encoded_value)?;
    println!("{decoded_value}");
    Ok(())
}

fn execute_command_info(file_path: &str) -> anyhow::Result<()> {
    let client = TorrentClient::from_torrent_file(file_path)?;
    let torrent = client.torrent_metainfo;
    println!("Tracker URL: {}", torrent.announce);
    println!("Length: {}", torrent.info.length);
    println!("Info Hash: {}", torrent.info.hash_hex()?);

    println!("Piece Length: {}", torrent.info.piece_length);
    println!("Piece Hashes:");
    torrent
        .info
        .pieces_hashes()
        .iter()
        .for_each(|hash| println!("{hash}"));
    Ok(())
}

async fn execute_command_peers(file_path: &str) -> anyhow::Result<()> {
    let mut client = TorrentClient::from_torrent_file(file_path)?;
    client.fetch_peers().await?;
    client.peers.iter().for_each(|peer| println!("{peer}"));
    Ok(())
}

async fn execute_command_handshake(file_path: &str) -> anyhow::Result<()> {
    let mut client = TorrentClient::from_torrent_file(file_path)?;
    client.fetch_peers().await?;
    client.connect().await?;
    let peer_id = client.handshake().await?;
    println!("Peer ID: {peer_id}");
    Ok(())
}

async fn execute_command_download_piece(
    input_file_path: &str,
    output_file_path: &str,
    piece_index: u32,
) -> anyhow::Result<()> {
    let mut client = TorrentClient::from_torrent_file(input_file_path)?;
    client.fetch_peers().await?;
    client.connect().await?;
    client.handshake().await?;
    client.download_piece(piece_index.clone()).await?;
    std::fs::write(&output_file_path, &client.pieces[piece_index as usize])?;
    client.disconnect().await?;
    Ok(())
}

async fn execute_command_download(
    input_file_path: &str,
    output_file_path: &str,
) -> anyhow::Result<()> {
    let mut client = TorrentClient::from_torrent_file(input_file_path)?;
    client.fetch_peers().await?;
    client.connect().await?;
    client.handshake().await?;
    client.download().await?;
    client.save(output_file_path).await?;
    client.disconnect().await?;
    Ok(())
}
