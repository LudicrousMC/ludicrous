use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct TeleportEntityPayload {
    pub e_id: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vel_x: f64,
    pub vel_y: f64,
    pub vel_z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

pub struct TeleportEntity {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    payload: TeleportEntityPayload,
}

impl TeleportEntity {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        payload: TeleportEntityPayload,
    ) -> Self {
        TeleportEntity {
            player,
            socket_write,
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Packet for TeleportEntity {
    const CLIENTBOUND_ID: i32 = 0x1F;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut teleport_entity_packet = write_varint(Self::CLIENTBOUND_ID);
        teleport_entity_packet.extend(write_varint(self.payload.e_id)); // entity id
        teleport_entity_packet.extend(self.payload.x.to_be_bytes()); // x
        teleport_entity_packet.extend(self.payload.y.to_be_bytes()); // y
        teleport_entity_packet.extend(self.payload.z.to_be_bytes()); // z
        teleport_entity_packet.extend(self.payload.vel_x.to_be_bytes()); // vel x
        teleport_entity_packet.extend(self.payload.vel_y.to_be_bytes()); // vel y
        teleport_entity_packet.extend(self.payload.vel_z.to_be_bytes()); // vel z
        teleport_entity_packet.extend(self.payload.yaw.to_be_bytes()); // yaw
        teleport_entity_packet.extend(self.payload.pitch.to_be_bytes()); // pitch
        teleport_entity_packet.push(self.payload.on_ground as u8);
        self.format_packet(&mut teleport_entity_packet);
        self.encrypt_packet(&mut teleport_entity_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&teleport_entity_packet)
            .await
            .unwrap();
    }
}
