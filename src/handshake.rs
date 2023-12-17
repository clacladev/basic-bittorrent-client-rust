use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// Perform an handshake with a peer and receives back a peer ID
pub async fn handshake(peer: &str, info_hash: &[u8]) -> io::Result<String> {
    // Connect to the peer
    let mut stream = TcpStream::connect(peer).await?;

    // Prepare the handshake message
    let mut message = [0; 68];
    message[0] = 19; // Length of the protocol string
    message[1..20].copy_from_slice(b"BitTorrent protocol"); // Protocol string
                                                            // The next 8 bytes are already set to zero by default
    message[28..48].copy_from_slice(info_hash); // The next 20 bytes are the sha1 infohash
    message[48..68].copy_from_slice(b"00112233445566778899"); // The next 20 bytes are the peer id

    // Send the handshake message
    stream.write_all(&message).await?;

    // Receive a response
    let mut buffer = [0; 68];
    stream.read(&mut buffer).await?;

    // Extract the peer ID from the received message
    let peer_id = hex::encode(&buffer[48..68]);

    Ok(peer_id.into())
}
