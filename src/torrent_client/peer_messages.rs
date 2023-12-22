#[derive(Debug)]
pub enum PeerMessageId {
    Bitfield,
    // Interested,
}

impl PeerMessageId {
    fn from(numeric_id: u8) -> anyhow::Result<Self> {
        match numeric_id {
            5 => Ok(Self::Bitfield),
            _ => Err(anyhow::Error::msg(
                format!("Unrecognised peer message with id '{numeric_id}'").to_string(),
            )),
        }
    }
}

#[derive(Debug)]
pub struct PeerMessage {
    pub id: PeerMessageId,
    pub length: u32,
    pub payload: Vec<u8>,
}

impl PeerMessage {
    fn new(id: PeerMessageId, length: u32, payload: Vec<u8>) -> Self {
        Self {
            id,
            length,
            payload,
        }
    }
}

impl PeerMessage {
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        println!("from_bytes :: len: {}", bytes.len());

        let message_length = u32::from_be_bytes(bytes[0..4].try_into()?);
        let message_id = PeerMessageId::from(bytes[5])?;

        let mut payload: Vec<u8> = vec![];
        if bytes.len() > 5 {
            payload = bytes[6..].to_vec();
        }

        let bytes_length = bytes.len();
        println!(
            "from_bytes :: message_length: {message_length}; message_id: {:?}: bytes_length: {bytes_length}",
            message_id
        );

        Ok(PeerMessage::new(message_id, message_length, payload))
    }
}
