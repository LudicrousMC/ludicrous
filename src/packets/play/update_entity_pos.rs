use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct UpdateEntityPosPayload {
    pub e_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub on_ground: bool,
}

/**
    Update Entity Position Packet
    Updates the position of an entity on the client

    # Clientbound
        * id: `0x2E`
        * resource: `move_entity_pos`

    # Serverbound
        * No relevant serverbound packet
        * Indirectly used by `Set Player Position` packet (set_player_pos.rs)
*/
pub struct UpdateEntityPos {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    payload: UpdateEntityPosPayload,
}

impl UpdateEntityPos {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        payload: UpdateEntityPosPayload,
    ) -> Self {
        UpdateEntityPos {
            player,
            socket_write,
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Packet for UpdateEntityPos {
    const CLIENTBOUND_ID: i32 = 0x2E;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    fn socket_write(&self) -> Option<Arc<AMutex<OwnedWriteHalf>>> {
        Some(self.socket_write.clone())
    }

    async fn send(&self) {
        let mut update_pos_packet = write_varint(Self::CLIENTBOUND_ID);
        update_pos_packet.extend(write_varint(self.payload.e_id)); // entity id
        update_pos_packet.extend(self.payload.delta_x.to_be_bytes()); // delta x
        update_pos_packet.extend(self.payload.delta_y.to_be_bytes()); // delta y
        update_pos_packet.extend(self.payload.delta_z.to_be_bytes()); // delta z
        update_pos_packet.push(self.payload.on_ground as u8); // on ground bool
        self.write_packet(update_pos_packet).await;
        /*
        println!(
            "update entity: {}, user: {}, dx: {}, dy: {}, dz: {}",
            self.payload.e_id,
            self.player.lock().await.uuid[15],
            self.payload.delta_x,
            self.payload.delta_y,
            self.payload.delta_z
        );*/
    }
}
