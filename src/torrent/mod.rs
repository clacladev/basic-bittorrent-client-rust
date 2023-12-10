use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct TorrentMetainfo {
    pub announce: String,
    pub info: Info,
}

#[derive(Serialize, Deserialize)]
pub struct Info {
    pub length: usize,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

impl Info {
    pub fn hash(&self) -> anyhow::Result<String> {
        let mut hasher = Sha1::new();
        let bytes = serde_bencode::to_bytes(self)?;
        hasher.update(bytes);
        let raw_hash = hasher.finalize();
        let hash = format!("{:x}", raw_hash);
        Ok(hash)
    }
}

pub fn decode_torrent_file(file_path: &str) -> anyhow::Result<TorrentMetainfo> {
    let content = fs::read(file_path)?;
    let torrent: TorrentMetainfo = serde_bencode::from_bytes(&content)?;
    Ok(torrent)
}
