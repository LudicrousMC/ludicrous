pub mod configuration;
pub mod handshake;
pub mod login;
pub mod play;
pub mod status;
use super::player::{Player, PlayerReadConn, PlayerStream, PlayerWriteConn};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use rsa::RsaPrivateKey;
use std::io::{Read, Write};
use std::sync::{atomic::Ordering, Arc, OnceLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex as AMutex;

pub struct EncryptionData {
    pub public_key: Vec<u8>,
    pub private_key: RsaPrivateKey,
}
pub static ENCRYPTION_DATA: OnceLock<EncryptionData> = OnceLock::new();

pub trait PacketStatic {
    /**
        # `Clientbound` packet id
        Value is the same as `serverbound` id unless overridden

        *If a clientbound packet id is not defined, then a serverbound packet id must be defined to
        prevent a circular reference*
    */
    const CLIENTBOUND_ID: i32 = Self::SERVERBOUND_ID;

    /**
        # `Serverbound` packet id
        Value is the same as `clientbound` id unless overridden

        *If a serverbound packet id is not defined, then a clientbound packet id must be defined to
        prevent a circular reference*
    */
    const SERVERBOUND_ID: i32 = Self::CLIENTBOUND_ID;

    /**
     *   # Packet Mode
     *   Describes the packets transmission between the server and client
     *   - Send: Packet is only sent to the client
     *   - Recieve: Packet is only received from the client
     *   - SendThenReceive: Packet is sent to the client then received from the client
     *   - ReceiveThenSend: Packet is recieved from the client and then sent to the client
     */
    const PACKET_MODE: PacketMode = PacketMode::Send;
}

pub enum PacketMode {
    Send,
    Receive,
    ReceiveThenSend,
    SendThenReceive,
}

/// TODO: Make comments for this trait regarding mathods and packet ids
#[async_trait::async_trait]
pub trait Packet: Send + Sync {
    /// The mode of the packet as defined in PacketStatic trait or else Send
    fn mode(&self) -> PacketMode {
        PacketMode::Send
    }

    /// Send a `clientbound (S -> C)` packet of self
    ///
    /// Uses the given player's write socket to send packet data to the client
    async fn send(&mut self, _write_conn: &mut PlayerWriteConn) {}

    /// Handle a `serverbound (C -> S)` packet of self
    ///
    /// Uses the given player's read socket to receive packet data and handle it accordingly
    async fn receive(&mut self, _read_conn: &mut PlayerReadConn) {}

    async fn receive_then_send(&mut self, conn: &mut PlayerStream) {
        self.receive(&mut conn.read).await;
        self.send(&mut conn.write).await;
    }

    async fn send_then_receive(&mut self, conn: &mut PlayerStream) {
        self.send(&mut conn.write).await;
        self.receive(&mut conn.read).await;
    }

    /// Handles a packet according to that packet's mode
    async fn handle(&mut self, conn: &mut PlayerStream) {
        match self.mode() {
            PacketMode::Send => self.send(&mut conn.write).await,
            PacketMode::Receive => self.receive(&mut conn.read).await,
            PacketMode::ReceiveThenSend => self.receive_then_send(conn).await,
            PacketMode::SendThenReceive => self.send_then_receive(conn).await,
        }
    }
}

impl<T: Packet + 'static> From<T> for Box<dyn Packet> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl PlayerReadConn {
    /// Decrypt packet of self
    pub async fn decrypt_packet(&mut self) -> Option<Vec<u8>> {
        if self.decryptor.is_some() {
            let mut packet_payload = self.decrypt_data().await?;
            if self
                .data
                .clone()
                .unwrap()
                .compression_enabled
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let data_length = read_varint_from_vec(&mut packet_payload);
                if data_length.is_none() {
                    return Some(Vec::new());
                }
                let data_length = data_length.unwrap();
                if data_length == 0 {
                    Some(packet_payload)
                } else {
                    let mut decompressed = vec![];
                    let _ =
                        ZlibDecoder::new(packet_payload.as_slice()).read_to_end(&mut decompressed);
                    Some(decompressed)
                }
            } else {
                Some(packet_payload)
            }
        } else {
            let packet_length = self.read_varint().await;
            let mut packet_payload = vec![0u8; packet_length as usize];
            self.socket_read
                .read_exact(&mut packet_payload)
                .await
                .unwrap();
            if self
                .data
                .clone()
                .unwrap()
                .compression_enabled
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                let data_length = read_varint_from_vec(&mut packet_payload).unwrap();
                if data_length == 0 {
                    Some(packet_payload)
                } else {
                    let mut decompressed = vec![];
                    ZlibDecoder::new(packet_payload.as_slice())
                        .read_to_end(&mut decompressed)
                        .unwrap();
                    Some(decompressed)
                }
            } else {
                Some(packet_payload)
            }
        }
    }

    async fn decrypt_data(&mut self) -> Option<Vec<u8>> {
        let decryptor = self.decryptor.as_mut()?;
        loop {
            // Validate packet
            // if received packet length is greater than or equal to expected length, return packet
            // if it's less than, then the whole packet is not received yet so continue reading
            if let Some((packet_len, len_bytes)) =
                peek_varint_and_len_from_slice(&self.decrypted_data)
            {
                let total_pkt_len = packet_len as usize + len_bytes;
                if self.decrypted_data.len() >= total_pkt_len {
                    // get packet data
                    let pkt = self.decrypted_data[len_bytes..total_pkt_len].to_vec();
                    // remove packet from temporary storage
                    self.decrypted_data.drain(..total_pkt_len);
                    // return packet data
                    break Some(pkt);
                }
            }

            let n = self
                .socket_read
                .read(&mut self.encrypted_buf)
                .await
                .unwrap();
            if n == 0 {
                break None;
            }

            let decrypt_n = decryptor.update(&self.encrypted_buf[..n], &mut self.decrypted_buf);
            if let Ok(decrypt_n) = decrypt_n {
                self.decrypted_data
                    .extend_from_slice(&self.decrypted_buf[..decrypt_n]);
            } else {
                break None;
            }
        }
    }

    pub async fn read_varint(&mut self) -> i32 {
        let mut value = 0;
        let mut position = 0;
        let mut current_byte: u8;

        loop {
            current_byte = self.socket_read.read_u8().await.unwrap_or(0x00);
            value |= (current_byte as i32 & 0x7F) << position;

            if current_byte & 0x80 == 0 {
                break;
            }

            position += 7;

            if position >= 32 {
                break;
            }
        }
        value
    }
}

impl PlayerWriteConn {
    /**
        Write a packet to the TCP Stream
    */
    async fn write_packet(&mut self, mut packet: Vec<u8>) -> std::io::Result<()> {
        let mut header = create_pkt_header(&mut packet, self.data.clone());
        self.encrypt_packet(&mut header);
        self.socket_write.write_all(&header).await?;
        self.encrypt_packet(&mut packet);
        self.socket_write.write_all(&packet).await?;
        Ok(())
    }

    async fn write_packet_data(&mut self, mut formatted_packet: Vec<u8>) -> std::io::Result<()> {
        self.encrypt_packet(&mut formatted_packet);
        self.socket_write.write_all(&formatted_packet).await?;
        Ok(())
    }

    fn encrypt_packet(&mut self, data: &mut [u8]) {
        if let Some(encryptor) = &mut self.encryptor {
            self.encrypt_buf.resize(data.len(), 0);
            encryptor.update(data, &mut self.encrypt_buf).unwrap();
            data.copy_from_slice(&self.encrypt_buf);
        }
    }
}

/// Compresses given packet and returns the previous length as varint if at compression
/// threshold, 0 if below threshold, or None if compression disabled
fn compress_packet(payload: &mut Vec<u8>, player_data: Arc<Player>) -> Option<Vec<u8>> {
    if player_data.compression_enabled.load(Ordering::Relaxed) {
        if payload.len() >= player_data.server.config.network_compression_threshold as usize {
            let pkt_len = payload.len() as i32;
            let mut encoder = ZlibEncoder::new(Vec::new(), flate2::Compression::new(3));
            encoder.write_all(payload).unwrap();
            *payload = encoder.finish().unwrap();
            Some(write_varint(pkt_len))
        } else {
            Some(write_varint(0))
        }
    } else {
        None
    }
}

/// Compresses a packet if applicable and returns packet's header
fn create_pkt_header(pkt: &mut Vec<u8>, player_data: Option<Arc<Player>>) -> Vec<u8> {
    let mut header = Vec::new();
    let header_pkt = if let Some(player_data) = player_data {
        compress_packet(pkt, player_data)
    } else {
        None
    };

    let mut total_len = pkt.len();
    if let Some(header_pkt) = &header_pkt {
        total_len += header_pkt.len();
    }
    header.extend(write_varint(total_len as i32));
    if let Some(header_pkt) = header_pkt {
        header.extend(header_pkt);
    }
    header
}

fn prepend_varint(payload: &mut Vec<u8>, num: i32) {
    payload.splice(0..0, write_varint(num));
}

fn prepend_len_as_varint(data: &mut Vec<u8>) {
    data.splice(0..0, write_varint(data.len() as i32));
}

pub fn read_varint_from_vec(bytes: &mut Vec<u8>) -> Option<i32> {
    let (data, varint_len) = peek_varint_and_len_from_slice(bytes)?;
    bytes.drain(..varint_len);
    Some(data)
}

/// Converts the first bytes of an array to a varint and returns that integer plus varint length
pub fn peek_varint_and_len_from_slice(bytes: &[u8]) -> Option<(i32, usize)> {
    let mut value = 0;
    let mut position = 0;
    let mut byte_positon = 0;
    let mut current_byte;

    while byte_positon < bytes.len() {
        current_byte = bytes[byte_positon];
        value |= (current_byte as i32 & 0x7F) << position;

        if current_byte & 0x80 == 0 {
            return Some((value, byte_positon + 1));
        }

        position += 7;
        if position >= 32 {
            return None;
        }
        byte_positon += 1;
    }
    None
}

pub async fn read_varlong(socket_ref: Arc<AMutex<TcpStream>>) -> Option<i64> {
    let mut value = 0i64;
    let mut position = 0;
    let mut current_byte;

    loop {
        current_byte = socket_ref.lock().await.read_u8().await.unwrap();
        value |= (current_byte as i64 & 0x7F) << position;

        if current_byte & 0x80 == 0 {
            break Some(value);
        }

        position += 7;

        if position >= 64 {
            break None;
        }
    }
}

pub fn write_varint(value: i32) -> Vec<u8> {
    let mut value = value as u32;
    let mut buf = Vec::new();
    loop {
        if (value & !0x7F) == 0 {
            buf.push(value as u8);
            break buf;
        }
        buf.push(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
}

pub async fn write_varlong(mut value: i64) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let mut temp = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            temp |= 0x80;
        }
        buf.push(temp);
        if value == 0 {
            break;
        }
    }
    buf
}

pub fn write_string(s: &str) -> Vec<u8> {
    let mut buf = write_varint(s.len() as i32);
    buf.extend(s.as_bytes());
    buf
}
