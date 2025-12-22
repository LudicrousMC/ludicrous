use super::super::{
    write_string, write_varint, Packet, PacketMode, PacketStatic, PlayerReadConn, PlayerWriteConn,
};
use crate::{MC_PROTOCOL, MC_VERSION};
use serde_json::json;

pub struct Status;

impl Status {
    pub fn new() -> Self {
        Status
    }
}

impl PacketStatic for Status {
    const SERVERBOUND_ID: i32 = 0x00;
    const PACKET_MODE: PacketMode = PacketMode::ReceiveThenSend;
}

#[async_trait::async_trait]
impl Packet for Status {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        let _status_packet_len = read_conn.read_varint().await;
        let _status_packet_id = read_conn.read_varint().await;

        // Empty packet body
    }

    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        // Server Status Response Packet (s -> c)
        let status_response = json!({
            "version": {"name": MC_VERSION, "protocol": MC_PROTOCOL},
            "players": {
                "max": 100,
                "online": 0,
                "sample": []
            },
            "description": {"text": "Ludicrous Dev Server"}
        });
        let mut status_packet = write_varint(Self::CLIENTBOUND_ID);
        status_packet.extend(write_string(&status_response.to_string()));
        write_conn.write_packet(status_packet).await;
    }
}
