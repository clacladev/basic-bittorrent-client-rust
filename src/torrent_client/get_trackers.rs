use serde::{Deserialize, Serialize};

use super::torrent_metainfo::TorrentMetainfo;

#[derive(Debug)]
pub struct GetTrackersRequest {
    pub peer_id: String,
    pub torrent: TorrentMetainfo,
}

impl GetTrackersRequest {
    pub fn new(peer_id: &str, torrent: TorrentMetainfo) -> Self {
        Self {
            peer_id: peer_id.into(),
            torrent,
        }
    }
}

impl GetTrackersRequest {
    pub fn to_url(&self) -> anyhow::Result<String> {
        let params = vec![
            ("peer_id", self.peer_id.to_string()),
            ("port", "6881".to_string()),
            ("uploaded", "0".to_string()),
            ("downloaded", "0".to_string()),
            ("left", format!("{}", self.torrent.info.length)),
            ("compact", "1".to_string()),
        ];
        let encoded_params = serde_urlencoded::to_string(params)?;
        let info_hash = self.torrent.info.hash_string()?;

        let url = format!(
            "{}?info_hash={}&{}",
            self.torrent.announce, info_hash, encoded_params
        );

        Ok(url)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTrackersResponse {
    #[serde(rename = "peers", with = "serde_bytes")]
    pub raw_peers_string: Vec<u8>,
}

impl GetTrackersResponse {
    pub fn peers(&self) -> Vec<String> {
        return self
            .raw_peers_string
            .chunks_exact(6)
            .map(|chunk| {
                let port: u16 = ((chunk[4] as u16) << 8) | (chunk[5] as u16);
                return format!(
                    "{}.{}.{}.{}:{}",
                    chunk[0], chunk[1], chunk[2], chunk[3], port
                );
            })
            .collect();
    }
}
