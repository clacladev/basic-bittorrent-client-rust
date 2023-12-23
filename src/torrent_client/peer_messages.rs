use crate::torrent_client::error::Error;

#[derive(Debug)]
pub enum PeerMessageId {
    Interested,
    Bitfield,
}

impl PeerMessageId {
    pub fn from(numeric_id: u8) -> anyhow::Result<Self> {
        match numeric_id {
            2 => Ok(Self::Interested),
            5 => Ok(Self::Bitfield),
            id => Err(anyhow::Error::msg(
                Error::PeerMessageIdNotRecognized(id).to_string(),
            )),
        }
    }

    pub fn to_numeric(&self) -> u8 {
        match self {
            Self::Interested => 2,
            Self::Bitfield => 5,
        }
    }
}

#[derive(Debug)]
pub enum PeerMessage {
    Interested,
    Bitfield(u8), // bitfield byte
}

impl PeerMessage {
    pub fn id(&self) -> PeerMessageId {
        match self {
            Self::Interested => PeerMessageId::Interested,
            Self::Bitfield(_) => PeerMessageId::Bitfield,
        }
    }
}

impl PeerMessage {
    pub fn from_bytes(numeric_id: u8, body: &[u8]) -> anyhow::Result<Self> {
        let peer_message_id = PeerMessageId::from(numeric_id)?;
        match peer_message_id {
            PeerMessageId::Interested => Ok(Self::Interested),
            PeerMessageId::Bitfield => Ok(Self::Bitfield(body[0])),
        }
    }

    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::Interested => {
                let message_length: u32 = 1;
                let mut bytes = vec![0u8; 5];
                bytes[0..4].copy_from_slice(&message_length.to_be_bytes());
                bytes[4] = self.id().to_numeric();
                Some(bytes)
            }
            _ => None,
        }
    }
}
