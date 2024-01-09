use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    vec,
};

use hex::ToHex;
use sha1::{Digest, Sha1};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub mod error;
mod get_trackers;
mod handshake_message;
mod peer_message;
mod torrent_metainfo;

use self::get_trackers::{GetTrackersRequest, GetTrackersResponse};
use self::handshake_message::HandshakeMessage;
use self::peer_message::PeerMessage;
use self::{error::Error, torrent_metainfo::TorrentMetainfo};

const PEER_ID: &str = "00112233445566778899";
const PIECE_BLOCK_SIZE: u32 = 16_384; // 16 KiB

pub struct TorrentClient {
    pub torrent_metainfo: TorrentMetainfo,
    pub peers: Vec<SocketAddr>,
    pub stream: Option<TcpStream>,
    pieces_bytes: Vec<Vec<u8>>,
}

// New and from helpers
impl TorrentClient {
    pub fn new(torrent_metainfo: TorrentMetainfo) -> Self {
        Self {
            torrent_metainfo,
            peers: vec![],
            stream: None,
            pieces_bytes: vec![],
        }
    }

    pub fn from_torrent_file(file_path: &str) -> anyhow::Result<Self> {
        let content = fs::read(file_path)?;
        let torrent_metainfo: TorrentMetainfo = serde_bencode::from_bytes(&content)?;
        Ok(Self::new(torrent_metainfo))
    }
}

// Peers related
impl TorrentClient {
    pub async fn fetch_peers(&mut self) -> anyhow::Result<()> {
        let get_trackers_request = GetTrackersRequest::new(PEER_ID, self.torrent_metainfo.clone());
        let get_trackers_url = get_trackers_request.to_url()?;
        let response_bytes = reqwest::get(&get_trackers_url).await?.bytes().await?;
        let tracker_response: GetTrackersResponse = serde_bencode::from_bytes(&response_bytes)?;

        self.peers = tracker_response
            .peers()
            .iter()
            .filter_map(|peer_string| {
                let parts: Vec<&str> = peer_string.split(":").collect();
                if parts.len() != 2 {
                    return None;
                }
                let ip = match Ipv4Addr::from_str(parts[0]) {
                    Ok(ip) => IpAddr::V4(ip),
                    Err(_) => return None,
                };
                let port = match parts[1].parse() {
                    Ok(p) => p,
                    Err(_) => return None,
                };
                Some(SocketAddr::new(ip, port))
            })
            .collect();

        Ok(())
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let Some(peer_socket_address) = self.peers.first() else {
            return Err(anyhow::Error::msg(Error::NoPeerAvailable));
        };
        self.stream = Some(TcpStream::connect(peer_socket_address).await?);
        println!("> Connected to {peer_socket_address}");
        Ok(())
    }

    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        stream.flush().await?;
        stream.shutdown().await?;
        self.stream = None;

        println!("> Disconnected");
        Ok(())
    }

    pub async fn handshake(&mut self) -> anyhow::Result<String> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        let info_hash = self.torrent_metainfo.info.hash_bytes()?;

        // Prepare the handshake message
        let handshake_message = HandshakeMessage::new(info_hash.into(), PEER_ID.into());

        // Send the handshake message
        stream.write_all(&handshake_message.to_bytes()).await?;

        // Receive a response
        let mut buffer = [0; 68];
        stream.read(&mut buffer).await?;

        // Extract the peer ID from the received message
        let handshake_reply_message = HandshakeMessage::from_bytes(&buffer);
        let peer_id = handshake_reply_message.peer_id;

        println!("> Handshake successful (Peer ID: {peer_id})");
        Ok(peer_id)
    }

    pub async fn prepare_for_download(&mut self) -> anyhow::Result<()> {
        println!("> Preparing for download");

        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        loop {
            // Wait for the stream to be available
            stream.readable().await?;

            // Read a message
            let message = Self::read_message(stream).await?;
            println!("> Received message: {message}");

            // Actionate a received message if necessary
            match message {
                PeerMessage::Bitfield { .. } => {
                    // Send an interested message
                    Self::send_message(stream, PeerMessage::Interested).await?;
                }
                PeerMessage::Unchoke => {
                    // Success
                    break Ok(());
                }
                _ => {}
            }
        }
    }

    pub async fn download(&mut self) -> anyhow::Result<()> {
        let torrent_metainfo = self.torrent_metainfo.clone();
        let pieces_count = self.torrent_metainfo.info.pieces_count();
        println!("> Starting to download {pieces_count} pieces");

        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        for piece_index in 0..pieces_count {
            let piece_bytes =
                Self::download_piece(stream, &torrent_metainfo, piece_index as u32).await?;
            self.pieces_bytes.resize(piece_index as usize + 1, vec![]);
            self.pieces_bytes[piece_index as usize] = piece_bytes.clone();
        }

        println!("> Successfully downloaded file");
        Ok(())
    }

    pub async fn save(&mut self, output_file_path: &str) -> anyhow::Result<()> {
        let file_bytes = self.pieces_bytes.concat();
        std::fs::write(&output_file_path, file_bytes)?;
        Ok(())
    }
}

impl TorrentClient {
    pub async fn download_piece(
        stream: &mut TcpStream,
        torrent_metainfo: &TorrentMetainfo,
        piece_index: u32,
    ) -> anyhow::Result<Vec<u8>> {
        println!("> Starting to download piece {piece_index}");

        let piece_length = Self::get_piece_length(&torrent_metainfo, piece_index);
        println!("> Piece length: {piece_length} bytes");

        let mut piece_bytes: Vec<u8> = Vec::with_capacity(piece_length);

        // Send the first request message
        Self::send_download_piece_block_message(stream, piece_index, 0, piece_length).await?;

        loop {
            // Wait for the stream to be available
            stream.readable().await?;

            // Read a message
            let message = Self::read_message(stream).await?;
            println!("> Received message: {message}");

            let PeerMessage::Piece { begin, block, .. } = message else {
                continue;
            };

            // Append the block's bytes to the already downloaded bytes
            piece_bytes.extend(block.clone());

            // If it has downloaded all blocks in the piece, break the loop and end
            let begin_offset = begin as usize + block.len();
            if begin_offset >= piece_length {
                // Verify the piece
                let metainfo_piece_hash =
                    torrent_metainfo.info.pieces_hashes()[piece_index as usize].clone();
                Self::verify_piece(&piece_bytes, metainfo_piece_hash.as_str())?;

                // Finished
                println!("> Successfully downloaded piece {piece_index}");
                break Ok(piece_bytes);
            }

            // Send followup request messages
            Self::send_download_piece_block_message(
                stream,
                piece_index,
                begin_offset as u32,
                piece_length,
            )
            .await?;
        }
    }

    fn get_piece_length(torrent_metainfo: &TorrentMetainfo, piece_index: u32) -> usize {
        let file_length = torrent_metainfo.info.length;
        let piece_length = torrent_metainfo.info.piece_length;
        let pieces_count = torrent_metainfo.info.pieces_count();
        let piece_length = if piece_index == pieces_count as u32 - 1 {
            // Last piece
            file_length % piece_length as usize
        } else {
            piece_length
        };
        piece_length
    }

    async fn read_message(stream: &mut TcpStream) -> anyhow::Result<PeerMessage> {
        // Read the message size (first 4 bytes)
        let message_size = stream.read_u32().await;
        let Ok(message_size) = message_size else {
            return Err(anyhow::Error::msg(Error::PeerClosedConnection));
        };
        if message_size == 0 {
            return Err(anyhow::Error::msg(Error::PeerClosedConnection));
        }

        // Read the message id (following 1 byte)
        let message_id = stream.read_u8().await?;

        // Define the expected body length
        let expected_body_length =
            PeerMessage::get_expected_message_length(message_id, message_size as usize);
        let mut message_body = vec![0u8; expected_body_length];

        if expected_body_length > 0 {
            // Read the message body
            let read_body_length = stream.read_exact(&mut message_body).await?;
            if expected_body_length != read_body_length {
                println!(
                    "{}",
                    Error::MessageBodyNotReadCorrect {
                        expected: expected_body_length,
                        actual: read_body_length
                    }
                )
            }
        }

        // Return a peer message with the id and body read
        PeerMessage::from_bytes(message_id, &message_body)
    }

    async fn send_message(stream: &mut TcpStream, message: PeerMessage) -> anyhow::Result<()> {
        if let Some(message_bytes) = message.to_bytes() {
            stream.write_all(&message_bytes).await?;
            println!("> Sent message: {:?}", message);
        }
        Ok(())
    }

    async fn send_download_piece_block_message(
        stream: &mut TcpStream,
        piece_index: u32,
        begin_offset: u32,
        piece_length: usize,
    ) -> anyhow::Result<()> {
        let next_block_length = (piece_length as u32) - begin_offset;
        let next_block_length = if next_block_length > PIECE_BLOCK_SIZE {
            PIECE_BLOCK_SIZE
        } else {
            next_block_length
        };

        Self::send_message(
            stream,
            PeerMessage::Request {
                index: piece_index,
                begin: begin_offset,
                length: next_block_length,
            },
        )
        .await
    }

    fn verify_piece(piece_bytes: &[u8], metainfo_piece_hash: &str) -> anyhow::Result<()> {
        let mut hasher = Sha1::new();
        hasher.update(piece_bytes);
        let piece_hash: String = hasher.finalize().encode_hex::<String>();

        match piece_hash == metainfo_piece_hash {
            true => Ok(()),
            false => Err(anyhow::Error::msg(Error::PieceHashNotValid)),
        }
    }
}
