pub enum Command {
    Decode,
    Info,
    Peers,
    Handshake,
}

impl Command {
    pub fn from_str(string: &str) -> Option<Command> {
        return match string {
            "decode" => Some(Command::Decode),
            "info" => Some(Command::Info),
            "peers" => Some(Command::Peers),
            "handshake" => Some(Command::Handshake),
            _ => None,
        };
    }
}
