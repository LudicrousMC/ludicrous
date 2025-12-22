use super::super::{
    write_varint, Packet, PacketMode, PacketStatic, PlayerReadConn, PlayerWriteConn,
};
use tokio::io::AsyncReadExt;

pub struct Ping {
    timestamp: [u8; 8],
}

impl Ping {
    pub fn new() -> Self {
        Ping {
            timestamp: [0u8; 8],
        }
    }
}

impl PacketStatic for Ping {
    const SERVERBOUND_ID: i32 = 0x01;
    const PACKET_MODE: PacketMode = PacketMode::ReceiveThenSend;
}

#[async_trait::async_trait]
impl Packet for Ping {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        // Client Ping Request Packet (c -> s)
        read_conn
            .socket_read
            .read_exact(&mut self.timestamp)
            .await
            .unwrap();
    }

    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        // Server Ping Response Packet (s -> c)
        let mut pong_packet = write_varint(Self::CLIENTBOUND_ID);
        pong_packet.extend(self.timestamp);
        write_conn.write_packet(pong_packet).await;
    }
}
