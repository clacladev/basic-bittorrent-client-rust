use crate::torrent::TorrentMetainfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TrackerResponse {
    #[serde(rename = "peers")]
    pub raw_peers_string: String,
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
    println!("{url}");

    let response = reqwest::get(&url).await?.text().await?;
    println!("Response: {}", response);

    let bytes = reqwest::get(&url).await?.bytes().await?;
    let tracker_response: TrackerResponse = serde_bencode::from_bytes(&bytes)?;

    Ok(tracker_response)
}
