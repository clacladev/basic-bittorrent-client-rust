use crate::torrent_client::error::Error;

#[derive(Debug)]
pub enum PeerMessage {
    Unchoke,
    Interested,
    Bitfield(u8), // bitfield byte
}

impl PeerMessage {
    pub fn id(&self) -> u8 {
        match self {
            Self::Unchoke => 1,
            Self::Interested => 2,
            Self::Bitfield(_) => 5,
        }
    }
}

impl PeerMessage {
    pub fn from_bytes(id: u8, body: &[u8]) -> anyhow::Result<Self> {
        match id {
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            5 => Ok(Self::Bitfield(body[0])),
            _ => Err(anyhow::Error::msg(
                Error::PeerMessageIdNotRecognized(id).to_string(),
            )),
        }
    }

    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::Interested => {
                let message_length: u32 = 1;
                let mut bytes = vec![0u8; 5];
                bytes[0..4].copy_from_slice(&message_length.to_be_bytes());
                bytes[4] = self.id();
                Some(bytes)
            }
            _ => None,
        }
    }
}
