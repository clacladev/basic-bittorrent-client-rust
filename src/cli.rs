pub enum Command {
    Decode,
    Info,
    Peers,
    Handshake,
    DownloadPiece,
    Download,
}

impl Command {
    pub fn from_str(string: &str) -> Option<Command> {
        return match string {
            "decode" => Some(Command::Decode),
            "info" => Some(Command::Info),
            "peers" => Some(Command::Peers),
            "handshake" => Some(Command::Handshake),
            "download_piece" => Some(Command::DownloadPiece),
            "download" => Some(Command::Download),
            _ => None,
        };
    }
}
