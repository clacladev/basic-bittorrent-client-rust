use std::fmt;

#[derive(Debug)]
pub enum Error {
    NoPeerAvailable,
    TcpStreamNotAvailable,
    PeerClosedConnection,
    MessageBodyNotReadCorrect { expected: usize, actual: usize },
    PeerMessageIdNotRecognized { id: u8 },
    PieceHashNotValid,
    PieceNotSaved,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_message())
    }
}

impl Error {
    fn to_message(&self) -> String {
        match self {
            Self::NoPeerAvailable => "No peer available".into(),
            Self::TcpStreamNotAvailable => "Tcp stream not available".into(),
            Self::PeerClosedConnection => "Peer has closed connection".into(),
            Self::MessageBodyNotReadCorrect { expected, actual } => format!(
                "Message body was not read correct. Expected {expected} bytes, got {actual} bytes"
            ),
            Self::PeerMessageIdNotRecognized { id } => {
                format!("Peer message id '{}' not recognized", id)
            }
            Self::PieceHashNotValid => "Piece hash not valid".into(),
            Self::PieceNotSaved => "Piece not saved".into(),
        }
    }
}
