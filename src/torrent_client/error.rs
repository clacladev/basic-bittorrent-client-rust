pub enum Error {
    TcpStreamNotAvailable,
    MessageBodyNotReadCorrect,
    PeerMessageIdNotRecognized(u8),
}

impl Error {
    pub fn to_string(&self) -> String {
        match self {
            Self::TcpStreamNotAvailable => "Tcp stream not available".to_string(),
            Self::MessageBodyNotReadCorrect => "Message body was not read correct".to_string(),
            Self::PeerMessageIdNotRecognized(id) => {
                format!("Peer message id '{}' not recognized", id).to_string()
            }
        }
    }
}
