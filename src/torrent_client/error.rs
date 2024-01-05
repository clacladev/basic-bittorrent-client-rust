pub enum Error {
    NoPeerAvailable,
    TcpStreamNotAvailable,
    PeerClosedConnection,
    MessageBodyNotReadCorrect(usize, usize),
    PeerMessageIdNotRecognized(u8),
}

impl Error {
    pub fn to_string(&self) -> String {
        match self {
            Self::NoPeerAvailable => "No peer available".to_string(),
            Self::TcpStreamNotAvailable => "Tcp stream not available".to_string(),
            Self::PeerClosedConnection => "Peer has closed connection".to_string(),
            Self::MessageBodyNotReadCorrect(expected, actual) => format!(
                "Message body was not read correct. Expected {expected} bytes, got {actual} bytes"
            )
            .to_string(),
            Self::PeerMessageIdNotRecognized(id) => {
                format!("Peer message id '{}' not recognized", id).to_string()
            }
        }
    }
}
