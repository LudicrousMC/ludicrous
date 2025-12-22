use super::super::{write_varint, Packet, PacketStatic, Player, PlayerReadConn, PlayerWriteConn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

/**
    Serverbound Ping Packet
    Receives a serverbound ping and responds

    # Serverbound
    * id: `0x25`
    * resource: `ping_request`

    # Clientbound
    * id: `0x37`
    * resource: `pong_response`
*/
pub struct PingSB {
    packet_data: Vec<u8>,
}

impl PingSB {
    pub fn new(packet_data: Vec<u8>) -> Self {
        PingSB { packet_data }
    }
}

impl PacketStatic for PingSB {
    const SERVERBOUND_ID: i32 = 0x25;
    const CLIENTBOUND_ID: i32 = 0x37;
}

#[async_trait::async_trait]
impl Packet for PingSB {
    async fn receive(&mut self, _read_conn: &mut PlayerReadConn) {
        // Empty
    }

    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut ping_packet = write_varint(Self::CLIENTBOUND_ID);
        ping_packet.extend(&self.packet_data);
        write_conn.write_packet(ping_packet).await;
    }
}
