use tokio::net::TcpStream;

use crate::{
    torrent_file::{decode_torrent_file, TorrentMetainfo},
    tracker::tracker_get,
};

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
        if let None = self.stream {
            return Err(anyhow::Error::msg(Error::TcpStreamNotAvailable.to_string()));
        }

        let peer = self.peers.first().unwrap();
        self.stream = Some(TcpStream::connect(peer).await?);
        Ok(())
    }

    pub async fn download_piece(
        &self,
        piece_index: &u8,
        output_file_path: &str,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

// let info_hash = torrent.info.hash_bytes()?;
// let mut stream = TcpStream::connect(target_peer_address).await?;
// let _ = handshake(&mut stream, &info_hash).await?;
