use crate::torrent::TorrentMetainfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TrackerResponse {
    #[serde(rename = "peers", with = "serde_bytes")]
    pub raw_peers_string: Vec<u8>,
}

impl TrackerResponse {
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

pub async fn tracker_get(torrent: &TorrentMetainfo) -> anyhow::Result<TrackerResponse> {
    let params = vec![
        ("peer_id", "00112233445566778899".to_string()),
        ("port", "6881".to_string()),
        ("uploaded", "0".to_string()),
        ("downloaded", "0".to_string()),
        ("left", format!("{}", torrent.info.length)),
        ("compact", "1".to_string()),
    ];
    let encoded_params = serde_urlencoded::to_string(params)?;
    let info_hash = torrent.info.hash_string()?;

    let url = format!(
        "{}?info_hash={}&{}",
        torrent.announce, info_hash, encoded_params
    );

    let bytes = reqwest::get(&url).await?.bytes().await?;
    let tracker_response: TrackerResponse = serde_bencode::from_bytes(&bytes)?;

    Ok(tracker_response)
}
