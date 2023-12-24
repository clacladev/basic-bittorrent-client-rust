use crate::torrent_client::error::Error;

#[derive(Debug)]
pub enum PeerMessage {
    Unchoke,
    Interested,
    Bitfield(u8),             // bitfield byte
    Request(u32, u32, u32),   // index, begin, length
    Piece(u32, u32, Vec<u8>), // index, begin, block
}

impl PeerMessage {
    pub fn id(&self) -> u8 {
        match self {
            Self::Unchoke => 1,
            Self::Interested => 2,
            Self::Bitfield(_) => 5,
            Self::Request(_, _, _) => 6,
            Self::Piece(_, _, _) => 7,
        }
    }
}

impl PeerMessage {
    pub fn from_bytes(id: u8, body: &[u8]) -> anyhow::Result<Self> {
        match id {
            1 => Ok(Self::Unchoke),
            5 => Ok(Self::Bitfield(body[0])),
            7 => Self::get_piece_from_bytes(body),
            _ => Err(anyhow::Error::msg(
                Error::PeerMessageIdNotRecognized(id).to_string(),
            )),
        }
    }

    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::Interested => Some(Self::get_empty_message_bytes(self.id())),
            Self::Request(index, begin, length) => Some(Self::get_request_message_bytes(
                self.id(),
                *index,
                *begin,
                *length,
            )),
            _ => None,
        }
    }
}

impl PeerMessage {
    fn get_empty_message_bytes(id: u8) -> Vec<u8> {
        let message_length: u32 = 1;
        let mut bytes = vec![0u8; 5];
        bytes[0..4].copy_from_slice(&message_length.to_be_bytes());
        bytes[4] = id;
        bytes
    }

    fn get_request_message_bytes(id: u8, index: u32, begin: u32, length: u32) -> Vec<u8> {
        let message_length: u32 = 13;
        let mut bytes = vec![0u8; 17];
        bytes[0..4].copy_from_slice(&message_length.to_be_bytes());
        bytes[4] = id;
        bytes[5..9].copy_from_slice(&index.to_be_bytes());
        bytes[9..13].copy_from_slice(&begin.to_be_bytes());
        bytes[13..17].copy_from_slice(&length.to_be_bytes());
        bytes
    }

    fn get_piece_from_bytes(bytes: &[u8]) -> anyhow::Result<PeerMessage> {
        let index = u32::from_be_bytes(bytes[0..4].try_into()?);
        let begin = u32::from_be_bytes(bytes[4..8].try_into()?);
        let block = bytes[8..].to_vec();
        Ok(Self::Piece(index, begin, block))
    }
}
