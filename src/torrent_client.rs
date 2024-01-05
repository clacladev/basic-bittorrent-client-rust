use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod error;
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
    stream: Option<TcpStream>,
}

// New and from helpers
impl TorrentClient {
    pub fn new(torrent_metainfo: TorrentMetainfo) -> Self {
        Self {
            torrent_metainfo,
            peers: vec![],
            stream: None,
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
}

// Peer connection related
impl TorrentClient {
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let Some(peer_socket_address) = self.peers.first() else {
            return Err(anyhow::Error::msg(Error::NoPeerAvailable));
        };
        self.stream = Some(TcpStream::connect(peer_socket_address).await?);
        println!("> Connected");
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

        println!("> Handshake successful (Peer ID: {})", peer_id);
        Ok(peer_id)
    }

    pub async fn download_piece(
        &mut self,
        piece_index: u32,
        // output_file_path: &str,
    ) -> anyhow::Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        loop {
            // Wait for the stream to be available
            stream.readable().await?;

            // Read a message
            let message = Self::read_message(stream).await?;
            println!("> Received message: {:?}", message);

            // Actionate a message if necessary
            match message {
                PeerMessage::Bitfield { .. } => {
                    // Send an interested message
                    Self::send_message(stream, PeerMessage::Interested).await?;
                }
                PeerMessage::Unchoke => {
                    // Send the first request message
                    Self::send_download_piece_block_message(stream, piece_index, 0).await?;
                }
                PeerMessage::Piece { .. } => {
                    break Ok(());
                }
                // PeerMessage::Piece { index, begin, .. } => {
                //     // TODO: if index > total_num_pieces { save to file and close stream? }
                //     // Send followup request messages
                //     // Self::send_download_piece_block_message(stream, index, begin).await?;
                // }
                _ => {}
            }
        }
    }
}

impl TorrentClient {
    async fn read_message(stream: &mut TcpStream) -> anyhow::Result<PeerMessage> {
        // Read the message size (first 4 bytes)
        let message_size = stream.read_u32().await?;
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
                eprintln!(
                    "{}",
                    Error::MessageBodyNotReadCorrect(expected_body_length, read_body_length)
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
        block_index: u32,
    ) -> anyhow::Result<()> {
        // TODO: let message = PeerMessage::Request(piece_index, offset_begin, block_length);
        return Self::send_message(
            stream,
            PeerMessage::Request {
                index: piece_index,
                begin: block_index * PIECE_BLOCK_SIZE,
                length: PIECE_BLOCK_SIZE, // TODO: Use the correct block size
            },
        )
        .await;
    }
}
