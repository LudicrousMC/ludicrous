use super::{Packet, PacketMode, PacketStatic, PlayerReadConn};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, PartialEq)]
pub enum HandshakeState {
    Status,
    Login,
}

// Minecraft SLP (Server List Ping) Implementation
pub struct Handshake<'a> {
    state: &'a mut HandshakeState,
}

impl<'a> Handshake<'a> {
    pub fn new(state: &'a mut HandshakeState) -> Self {
        Handshake { state }
    }
}

impl PacketStatic for Handshake<'_> {
    const SERVERBOUND_ID: i32 = 0x00;
    const PACKET_MODE: PacketMode = PacketMode::Receive;
}

#[async_trait::async_trait]
impl Packet for Handshake<'_> {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        // Initial Handshake (c -> s)
        let _handshake_packet_len = read_conn.read_varint().await;
        let _handshake_packet_id = read_conn.read_varint().await;
        let protocol = read_conn.read_varint().await;
        let mut addr_buf = vec![0u8; read_conn.read_varint().await as usize];
        read_conn
            .socket_read
            .read_exact(&mut addr_buf)
            .await
            .unwrap();
        let address = String::from_utf8_lossy(&addr_buf);
        let mut port_buf = [0u8; 2];
        read_conn
            .socket_read
            .read_exact(&mut port_buf)
            .await
            .unwrap();
        let port = u16::from_be_bytes(port_buf);
        let next_state = read_conn.read_varint().await;
        if next_state == 1 {
            *self.state = HandshakeState::Status;
        } else if next_state == 2 {
            *self.state = HandshakeState::Login;
        }
    }
}
