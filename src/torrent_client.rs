use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    vec,
};

use hex::ToHex;
use sha1::{Digest, Sha1};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

mod error;
mod get_trackers;
mod handshake_message;
mod peer_message;
mod torrent_metainfo;

use self::get_trackers::{GetTrackersRequest, GetTrackersResponse};
use self::handshake_message::HandshakeMessage;
use self::peer_message::PeerMessage;
use self::{error::Error, torrent_metainfo::TorrentMetainfo};

const PEER_ID: &str = "00112233445566778899";
const PIECE_BLOCK_SIZE: u32 = 16_384; // 16 KiB

pub struct TorrentClient {
    pub torrent_metainfo: TorrentMetainfo,
    pub peers: Vec<SocketAddr>,
    stream: Option<TcpStream>,
    pieces: Vec<Vec<u8>>,
}

// New and from helpers
impl TorrentClient {
    pub fn new(torrent_metainfo: TorrentMetainfo) -> Self {
        Self {
            torrent_metainfo,
            peers: vec![],
            stream: None,
            pieces: vec![],
        }
    }

    pub fn from_torrent_file(_file_path: &str) -> anyhow::Result<Self> {
        // let content = fs::read(file_path)?;

        // Note:
        // Hardcoded bytes of file /tmp/torrents2550026962/codercat.gif.torrent
        let content = [
            100, 56, 58, 97, 110, 110, 111, 117, 110, 99, 101, 53, 53, 58, 104, 116, 116, 112, 58,
            47, 47, 98, 105, 116, 116, 111, 114, 114, 101, 110, 116, 45, 116, 101, 115, 116, 45,
            116, 114, 97, 99, 107, 101, 114, 46, 99, 111, 100, 101, 99, 114, 97, 102, 116, 101,
            114, 115, 46, 105, 111, 47, 97, 110, 110, 111, 117, 110, 99, 101, 49, 48, 58, 99, 114,
            101, 97, 116, 101, 100, 32, 98, 121, 49, 51, 58, 109, 107, 116, 111, 114, 114, 101,
            110, 116, 32, 49, 46, 49, 52, 58, 105, 110, 102, 111, 100, 54, 58, 108, 101, 110, 103,
            116, 104, 105, 50, 53, 52, 57, 55, 48, 48, 101, 52, 58, 110, 97, 109, 101, 49, 52, 58,
            105, 116, 115, 119, 111, 114, 107, 105, 110, 103, 46, 103, 105, 102, 49, 50, 58, 112,
            105, 101, 99, 101, 32, 108, 101, 110, 103, 116, 104, 105, 50, 54, 50, 49, 52, 52, 101,
            54, 58, 112, 105, 101, 99, 101, 115, 50, 48, 48, 58, 1, 204, 23, 187, 230, 15, 165,
            165, 47, 100, 189, 95, 91, 100, 217, 146, 134, 213, 10, 165, 131, 143, 112, 60, 247,
            246, 240, 141, 28, 73, 126, 211, 144, 223, 120, 249, 13, 95, 117, 102, 69, 191, 16,
            151, 75, 88, 22, 73, 30, 48, 98, 139, 120, 163, 130, 202, 54, 196, 224, 95, 132, 190,
            75, 216, 85, 179, 75, 206, 220, 12, 110, 152, 246, 109, 62, 124, 99, 53, 61, 30, 134,
            66, 122, 201, 77, 110, 79, 33, 166, 208, 214, 200, 183, 255, 164, 195, 147, 195, 177,
            49, 124, 112, 205, 95, 68, 209, 172, 85, 5, 203, 133, 93, 82, 108, 235, 15, 95, 28,
            213, 227, 55, 150, 171, 5, 175, 31, 168, 116, 23, 58, 10, 108, 18, 152, 98, 90, 212,
            123, 79, 230, 39, 42, 143, 248, 252, 134, 91, 5, 61, 151, 74, 120, 104, 20, 20, 179,
            128, 119, 215, 177, 176, 113, 40, 211, 166, 1, 128, 98, 191, 231, 121, 219, 150, 211,
            169, 60, 5, 251, 129, 212, 122, 255, 201, 79, 9, 133, 185, 133, 235, 136, 138, 54, 236,
            146, 101, 40, 33, 162, 27, 228, 101, 101,
        ];
        println!("> Torrent metainfo file content: {:?}", content);

        let torrent_metainfo: TorrentMetainfo = serde_bencode::from_bytes(&content)?;
        println!("> Torrent metainfo: {:?}", torrent_metainfo);
        Ok(Self::new(torrent_metainfo))
    }
}

// Peers related
impl TorrentClient {
    pub async fn fetch_peers(&mut self) -> anyhow::Result<()> {
        let get_trackers_request = GetTrackersRequest::new(PEER_ID, self.torrent_metainfo.clone());
        let get_trackers_url = get_trackers_request.to_url()?;
        let response_bytes = reqwest::get(&get_trackers_url).await?.bytes().await?;
        let tracker_response: GetTrackersResponse = serde_bencode::from_bytes(&response_bytes)?;

        self.peers = tracker_response
            .peers()
            .iter()
            .filter_map(|peer_string| {
                let parts: Vec<&str> = peer_string.split(":").collect();
                if parts.len() != 2 {
                    return None;
                }
                let ip = match Ipv4Addr::from_str(parts[0]) {
                    Ok(ip) => IpAddr::V4(ip),
                    Err(_) => return None,
                };
                let port = match parts[1].parse() {
                    Ok(p) => p,
                    Err(_) => return None,
                };
                Some(SocketAddr::new(ip, port))
            })
            .collect();

        Ok(())
    }

    pub async fn connect(&mut self) -> anyhow::Result<()> {
        let Some(peer_socket_address) = self.peers.first() else {
            return Err(anyhow::Error::msg(Error::NoPeerAvailable));
        };
        self.stream = Some(TcpStream::connect(peer_socket_address).await?);
        println!("> Connected to {}", peer_socket_address);
        Ok(())
    }

    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        stream.flush().await?;
        stream.shutdown().await?;
        self.stream = None;

        println!("> Disconnected");
        Ok(())
    }

    pub async fn handshake(&mut self) -> anyhow::Result<String> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        let info_hash = self.torrent_metainfo.info.hash_bytes()?;

        // Prepare the handshake message
        let handshake_message = HandshakeMessage::new(info_hash.into(), PEER_ID.into());

        // Send the handshake message
        stream.write_all(&handshake_message.to_bytes()).await?;

        // Receive a response
        let mut buffer = [0; 68];
        stream.read(&mut buffer).await?;

        // Extract the peer ID from the received message
        let handshake_reply_message = HandshakeMessage::from_bytes(&buffer);
        let peer_id = handshake_reply_message.peer_id;

        println!("> Handshake successful (Peer ID: {})", peer_id);
        Ok(peer_id)
    }

    pub async fn download_piece(
        &mut self,
        piece_index: u32,
        output_file_path: &str,
    ) -> anyhow::Result<()> {
        let stream = self
            .stream
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg(Error::TcpStreamNotAvailable))?;

        let piece_length = self.torrent_metainfo.info.piece_length;
        let mut piece_bytes: Vec<u8> = Vec::with_capacity(piece_length);
        println!("> Piece length: {}", piece_length);

        loop {
            // Wait for the stream to be available
            stream.readable().await?;

            // Read a message
            let message = Self::read_message(stream).await?;
            println!("> Received message: {}", message);

            // Actionate a received message if necessary
            match message {
                PeerMessage::Bitfield { .. } => {
                    // Send an interested message
                    Self::send_message(stream, PeerMessage::Interested).await?;
                }
                PeerMessage::Unchoke => {
                    // Send the first request message
                    Self::send_download_piece_block_message(stream, piece_index, 0, piece_length)
                        .await?;
                }
                PeerMessage::Piece { begin, block, .. } => {
                    // Append the block's bytes to the already downloaded bytes
                    piece_bytes.extend(block.clone());

                    // If it has downloaded all blocks in the piece, break the loop and end
                    let begin_offset = begin as usize + block.len();
                    if begin_offset >= piece_length {
                        // Save the piece bytes
                        self.pieces.resize(piece_index as usize + 1, vec![]);
                        self.pieces[piece_index as usize] = piece_bytes.clone();

                        // Verify the piece
                        self.verify_piece(piece_index)?;

                        // Save the piece to disk
                        std::fs::write(&output_file_path, piece_bytes)?;

                        // Finished
                        break Ok(());
                    }

                    // Send followup request messages
                    Self::send_download_piece_block_message(
                        stream,
                        piece_index,
                        begin_offset as u32,
                        piece_length,
                    )
                    .await?;
                }
                _ => {}
            }
        }
    }
}

// Helpers
impl TorrentClient {
    async fn read_message(stream: &mut TcpStream) -> anyhow::Result<PeerMessage> {
        // Read the message size (first 4 bytes)
        // Note:
        // Issue is here: it fails to read the first 4 bytes.
        // It throws an error: unexpected end of file
        let message_size = stream.read_u32().await?;
        if message_size == 0 {
            return Err(anyhow::Error::msg(Error::PeerClosedConnection));
        }

        // Read the message id (following 1 byte)
        let message_id = stream.read_u8().await?;

        // Define the expected body length
        let expected_body_length =
            PeerMessage::get_expected_message_length(message_id, message_size as usize);
        let mut message_body = vec![0u8; expected_body_length];

        if expected_body_length > 0 {
            // Read the message body
            let read_body_length = stream.read_exact(&mut message_body).await?;
            if expected_body_length != read_body_length {
                println!(
                    "{}",
                    Error::MessageBodyNotReadCorrect {
                        expected: expected_body_length,
                        actual: read_body_length
                    }
                )
            }
        }

        // Return a peer message with the id and body read
        PeerMessage::from_bytes(message_id, &message_body)
    }

    async fn send_message(stream: &mut TcpStream, message: PeerMessage) -> anyhow::Result<()> {
        if let Some(message_bytes) = message.to_bytes() {
            stream.write_all(&message_bytes).await?;
            println!("> Sent message: {:?}", message);
        }
        Ok(())
    }

    async fn send_download_piece_block_message(
        stream: &mut TcpStream,
        piece_index: u32,
        begin_offset: u32,
        piece_length: usize,
    ) -> anyhow::Result<()> {
        let next_block_length = (piece_length as u32) - begin_offset;
        let next_block_length = if next_block_length > PIECE_BLOCK_SIZE {
            PIECE_BLOCK_SIZE
        } else {
            next_block_length
        };

        Self::send_message(
            stream,
            // Note: here I hardcode the begin offset and length
            // to fetch the problematic block. So at the first read message it will fail.
            PeerMessage::Request {
                index: piece_index,
                begin: 180224,
                length: 16384,
            },
            // PeerMessage::Request {
            //     index: piece_index,
            //     begin: begin_offset,
            //     length: next_block_length,
            // },
        )
        .await
    }

    fn verify_piece(&self, piece_index: u32) -> anyhow::Result<()> {
        let piece_bytes = &self.pieces[piece_index as usize];

        let mut hasher = Sha1::new();
        hasher.update(piece_bytes);
        let piece_hash: String = hasher.finalize().encode_hex::<String>();

        let metainfo_piece_hash =
            self.torrent_metainfo.info.pieces_hashes()?[piece_index as usize].clone();

        if piece_hash != metainfo_piece_hash {
            return Err(anyhow::Error::msg(Error::PieceHashNotValid));
        }

        Ok(())
    }
}
