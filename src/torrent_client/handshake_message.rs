pub struct HandshakeMessage {
    pub info_hash: Vec<u8>,
    pub peer_id: String,
}

impl HandshakeMessage {
    pub fn new(info_hash: Vec<u8>, peer_id: String) -> Self {
        Self { info_hash, peer_id }
    }

    pub fn from_bytes(bytes: &[u8; 68]) -> Self {
        let info_hash = Vec::from(&bytes[28..48]);
        let peer_id = hex::encode(&bytes[48..68]);
        Self::new(info_hash, peer_id)
    }

    pub fn to_bytes(&self) -> [u8; 68] {
        let mut message = [0; 68];
        message[0] = 19; // Length of the protocol string
        message[1..20].copy_from_slice(b"BitTorrent protocol"); // Protocol string
                                                                // The next 8 bytes are already set to zero by default
        message[28..48].copy_from_slice(&self.info_hash[..]); // The next 20 bytes are the sha1 infohash
        message[48..68].copy_from_slice(self.peer_id.as_bytes()); // The next 20 bytes are the peer id
        return message;
    }
}
