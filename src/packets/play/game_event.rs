use super::super::{write_varint, Packet, PacketStatic, Player, PlayerWriteConn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct GameEvent {
    event_type: u8,
    event_value: f32,
}

impl GameEvent {
    pub fn new(event_type: u8, event_value: f32) -> Self {
        GameEvent {
            event_type,
            event_value,
        }
    }
}

impl PacketStatic for GameEvent {
    const CLIENTBOUND_ID: i32 = 0x22;
}

#[async_trait::async_trait]
impl Packet for GameEvent {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut game_event_packet = write_varint(Self::CLIENTBOUND_ID);
        game_event_packet.push(self.event_type); // game event
        game_event_packet.extend(self.event_value.to_be_bytes()); // float value
        write_conn.write_packet(game_event_packet).await;
    }
}
