use std::borrow::BorrowMut;

use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

mod peer_messages;

use crate::{
    handshake,
    torrent_file::{decode_torrent_file, TorrentMetainfo},
    tracker::tracker_get,
};

use self::peer_messages::PeerMessage;

pub enum Error {
    TcpStreamNotAvailable,
}

impl Error {
    pub fn to_string(&self) -> String {
        match self {
            Self::TcpStreamNotAvailable => "Tcp stream not available".to_string(),
        }
    }
}

pub struct TorrentClient {
    pub torrent_metainfo: TorrentMetainfo,
    pub peers: Vec<String>,
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
        let peers = tracker_response.peers();
        self.peers = peers;
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

        let mut buffer: Vec<u8> = vec![];
        let read_length = stream.read(&mut buffer).await?;

        // Tcp keepalive message to be ignored
        if read_length == 0 {
            println!("> keepalive");
            return Ok(());
        }

        let message = PeerMessage::from_bytes(buffer.as_slice())?;
        println!("> message: {:?}", message);

        Ok(())
    }
}
