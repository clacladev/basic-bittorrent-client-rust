use serde::{Deserialize, Serialize};
use serde_bencode;
use std::fs;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Info {
    pub length: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TorrentMetainfo {
    pub announce: String,
    pub info: Info,
}

pub fn decode_torrent_file(file_path: &str) -> anyhow::Result<TorrentMetainfo> {
    let content = fs::read(file_path)?;
    let torrent: TorrentMetainfo = serde_bencode::from_bytes(&content)?;
    Ok(torrent)
}
