use std::{
    borrow::BorrowMut,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod error;
mod handshake;
mod peer_messages;

use crate::{
    torrent_file::{decode_torrent_file, TorrentMetainfo},
    tracker::tracker_get,
};

use self::error::Error;
use self::peer_messages::PeerMessage;

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

    pub async fn from_torrent_file(file_path: &str) -> anyhow::Result<Self> {
        let torrent_metainfo = decode_torrent_file(file_path)?;
        Ok(Self::new(torrent_metainfo))
    }
}

// Peers related
impl TorrentClient {
    pub async fn fetch_peers(&mut self) -> anyhow::Result<()> {
        let tracker_response = tracker_get(&self.torrent_metainfo).await?;

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
        let peer = self.peers.first().unwrap();
        self.stream = Some(TcpStream::connect(peer).await?);
        println!("> Connected");
        Ok(())
    }

    pub async fn handshake(&mut self) -> anyhow::Result<String> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable.to_string()))?;

        let info_hash = self.torrent_metainfo.info.hash_bytes()?;
        let peer_id = handshake::handshake(stream.borrow_mut(), &info_hash).await?;
        println!("> Handshake successful (peer_id: {})", peer_id);
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
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable.to_string()))?;

        loop {
            // Wait for the stream to be available
            stream.readable().await?;

            // Read a message
            let message = Self::read_message(stream).await?;
            println!("> Received message: {:?}", message);

            // Actionate a message if necessary
            match message {
                PeerMessage::Bitfield(_) => {
                    // Send an interested message
                    Self::send_message(stream, PeerMessage::Interested).await?;
                }
                PeerMessage::Unchoke => {
                    // Send a request message
                    // TODO: Send a request message for each block of the piece
                    Self::send_message(
                        stream,
                        PeerMessage::Request(piece_index, 0, PIECE_BLOCK_SIZE),
                    )
                    .await?;
                }
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
            return Err(anyhow::Error::msg(Error::PeerClosedConnection.to_string()));
        }

        // Read the message id (following 1 byte)
        let message_id = stream.read_u8().await?;

        // Read the message body if necessary (following n bytes)
        let message_body_size = (message_size - 1) as usize;
        let mut message_body = vec![0u8; message_body_size];

        if message_body_size > 0 {
            // Read the
            let read_length = stream.read(&mut message_body).await?;
            if message_body_size != read_length {
                return Err(anyhow::Error::msg(
                    Error::MessageBodyNotReadCorrect.to_string(),
                ));
            }
        }

        // Make a peer message with the id and body read
        PeerMessage::from_bytes(message_id, message_body.as_slice())
    }

    async fn send_message(stream: &mut TcpStream, message: PeerMessage) -> anyhow::Result<()> {
        if let Some(message_bytes) = message.to_bytes() {
            stream.write_all(&message_bytes).await?;
            println!("> Sent message: {:?}", message);
        }
        Ok(())
    }
}
