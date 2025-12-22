use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;
use uuid::Uuid;

pub struct SpawnEntityPayload {
    pub e_id: i32,
    pub e_uuid: [u8; 16],
    pub e_type: i32,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub pitch: i8,
    pub yaw: i8,
    pub head_yaw: i8,
    pub data: i32,
    pub vel_x: i16,
    pub vel_y: i16,
    pub vel_z: i16,
}

/**
    Spawn Entity Packet
    Tells the client to spawn an entity

    # Clientbound
        * id: `0x01`
        * resource: `add_entity`

    # Serverbound
        No relevant serverbound packet
*/
pub struct SpawnEntity {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    payload: SpawnEntityPayload,
}

impl SpawnEntity {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        payload: SpawnEntityPayload,
    ) -> Self {
        SpawnEntity {
            player,
            socket_write,
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Packet for SpawnEntity {
    const CLIENTBOUND_ID: i32 = 0x01;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut spawn_entity_packet = write_varint(Self::CLIENTBOUND_ID);
        spawn_entity_packet.extend(write_varint(self.payload.e_id)); // entity id
        spawn_entity_packet.extend(self.payload.e_uuid); // entity uuid
        spawn_entity_packet.extend(write_varint(self.payload.e_type)); // entity type
        spawn_entity_packet.extend(self.payload.x.to_be_bytes()); // x
        spawn_entity_packet.extend(self.payload.y.to_be_bytes()); // y
        spawn_entity_packet.extend(self.payload.z.to_be_bytes()); // z
        spawn_entity_packet.extend(self.payload.pitch.to_be_bytes()); // pitch
        spawn_entity_packet.extend(self.payload.yaw.to_be_bytes()); // yaw
        spawn_entity_packet.extend(self.payload.head_yaw.to_be_bytes()); // head yaw
        spawn_entity_packet.extend(write_varint(self.payload.data)); // entity data
        spawn_entity_packet.extend(self.payload.vel_x.to_be_bytes()); // vel x
        spawn_entity_packet.extend(self.payload.vel_y.to_be_bytes()); // vel y
        spawn_entity_packet.extend(self.payload.vel_z.to_be_bytes()); // vel z
        self.format_packet(&mut spawn_entity_packet);
        self.encrypt_packet(&mut spawn_entity_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&spawn_entity_packet)
            .await
            .unwrap();
        /*println!(
            "new entity: {} {}",
            self.e_id,
            self.player.lock().await.uuid[15]
        );*/
    }
}
