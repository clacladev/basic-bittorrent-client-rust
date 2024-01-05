use crate::torrent_client::error::Error;

const PEER_MESSAGE_UNCHOKE_ID: u8 = 1;
const PEER_MESSAGE_INTERESTED_ID: u8 = 2;
const PEER_MESSAGE_BITFIELD_ID: u8 = 5;
const PEER_MESSAGE_REQUEST_ID: u8 = 6;
const PEER_MESSAGE_PIECE_ID: u8 = 7;

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
            Self::Unchoke => PEER_MESSAGE_UNCHOKE_ID,
            Self::Interested => PEER_MESSAGE_INTERESTED_ID,
            Self::Bitfield(_) => PEER_MESSAGE_BITFIELD_ID,
            Self::Request(_, _, _) => PEER_MESSAGE_REQUEST_ID,
            Self::Piece(_, _, _) => PEER_MESSAGE_PIECE_ID,
        }
    }
}

impl PeerMessage {
    pub fn from_bytes(id: u8, body: &[u8]) -> anyhow::Result<Self> {
        match id {
            PEER_MESSAGE_UNCHOKE_ID => Ok(Self::Unchoke),
            PEER_MESSAGE_BITFIELD_ID => Ok(Self::Bitfield(body[0])),
            PEER_MESSAGE_PIECE_ID => Self::get_piece_from_bytes(body),
            _ => Err(anyhow::Error::msg(Error::PeerMessageIdNotRecognized(id))),
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

impl PeerMessage {
    pub fn get_expected_message_length(id: u8, message_length_field_value: usize) -> usize {
        match id {
            PEER_MESSAGE_UNCHOKE_ID | PEER_MESSAGE_BITFIELD_ID | PEER_MESSAGE_PIECE_ID => {
                message_length_field_value - 1
            }
            _ => 0,
        }
    }
}
