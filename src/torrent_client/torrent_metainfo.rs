use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TorrentMetainfo {
    pub announce: String,
    pub info: Info,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Info {
    pub length: usize,
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
}

impl Info {
    pub fn hash_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut hasher = Sha1::new();
        let bytes = serde_bencode::to_bytes(self)?;
        hasher.update(bytes);
        let bytes = hasher.finalize();
        let bytes_vec = bytes.to_vec();
        Ok(bytes_vec)
    }

    pub fn hash_hex(&self) -> anyhow::Result<String> {
        let bytes = self.hash_bytes()?;
        let hash = bytes
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        Ok(hash)
    }

    pub fn hash_string(&self) -> anyhow::Result<String> {
        let bytes = self.hash_bytes()?;
        let mut str = String::new();
        for byte in bytes {
            str.push('%');
            str.push_str(&format!("{:02x}", byte));
        }
        Ok(str)
    }

    pub fn pieces_hashes(&self) -> anyhow::Result<Vec<String>> {
        let hashes: Vec<String> = self
            .pieces
            .chunks(20)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|&byte| format!("{:02x}", byte))
                    .collect::<String>()
            })
            .collect();
        Ok(hashes)
    }
}
