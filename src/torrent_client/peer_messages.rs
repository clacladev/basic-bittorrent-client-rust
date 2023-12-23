#[derive(Debug)]
pub enum PeerMessage {
    KeepAlive,
    Unknown(u8), // message_id
    Bitfield,
}

impl PeerMessage {
    fn from(numeric_id: u8) -> Self {
        match numeric_id {
            5 => Self::Bitfield,
            _ => Self::Unknown(numeric_id),
        }
    }
}

impl PeerMessage {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        println!(">> from_bytes len: {}", bytes.len());

        let message_length = u32::from_be_bytes(bytes[0..4].try_into()?);
        println!(">> message_length: {}", message_length);
        if bytes.len() == 4 && message_length == 0 {
            return Ok(PeerMessage::KeepAlive);
        }

        let message = PeerMessage::from(bytes[5]);
        Ok(message)
    }
}
