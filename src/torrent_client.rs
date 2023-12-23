use std::{
    borrow::BorrowMut,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use tokio::{io::AsyncReadExt, net::TcpStream};

mod peer_messages;

use crate::{
    handshake,
    torrent_file::{decode_torrent_file, TorrentMetainfo},
    tracker::tracker_get,
};

use self::peer_messages::PeerMessage;

pub enum Error {
    TcpStreamNotAvailable,
    MessageBodyNotReadCorrect,
}

impl Error {
    pub fn to_string(&self) -> String {
        match self {
            Self::TcpStreamNotAvailable => "Tcp stream not available".to_string(),
            Self::MessageBodyNotReadCorrect => "Message body was not read correct".to_string(),
        }
    }
}

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
        Ok(())
    }

    pub async fn handshake(&mut self) -> anyhow::Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable.to_string()))?;

        let info_hash = self.torrent_metainfo.info.hash_bytes()?;
        let _ = handshake(stream.borrow_mut(), &info_hash).await?;
        Ok(())
    }

    pub async fn download_piece(
        &mut self,
        // piece_index: &u8,
        // output_file_path: &str,
    ) -> anyhow::Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable.to_string()))?;

        loop {
            // Wait for the stream to be available
            stream.readable().await?;

            // Read the message size (first 4 bytes)
            let message_size = stream.read_u32().await?;
            if message_size == 0 {
                break; // The peer has closed the connection
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
            let peer_message = PeerMessage::from_bytes(message_id, message_body.as_slice())?;
            println!("> peer_message: {:?}", peer_message);
        }

        Ok(())
    }
}
