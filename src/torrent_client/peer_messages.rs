#[derive(Debug)]
pub enum PeerMessage {
    Unknown(u8), // message_id
    Bitfield(u8), // bitfield byte
                 // interested,
}

impl PeerMessage {
    pub fn from_bytes(id: u8, body: &[u8]) -> anyhow::Result<Self> {
        match id {
            5 => Ok(Self::Bitfield(body[0])),
            _ => Ok(Self::Unknown(id)),
        }
    }
}
