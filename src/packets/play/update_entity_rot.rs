use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct UpdateEntityRotPayload {
    pub e_id: i32,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

/**
    Update Entity Rotation Packet
    Updates the rotation of an entity on the client

    # Clientbound
        * id: `0x31`
        * resource: `move_entity_rot`

    # Serverbound
        * No relevant serverbound packet
        * Indirectly used by `Set Player Rotation` packet (set_player_rot.rs)
*/
pub struct UpdateEntityRot {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    payload: UpdateEntityRotPayload,
}

impl UpdateEntityRot {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        payload: UpdateEntityRotPayload,
    ) -> Self {
        UpdateEntityRot {
            player,
            socket_write,
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Packet for UpdateEntityRot {
    const CLIENTBOUND_ID: i32 = 0x31;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    fn socket_write(&self) -> Option<Arc<AMutex<OwnedWriteHalf>>> {
        Some(self.socket_write.clone())
    }

    async fn send(&self) {
        let mut update_rot_packet = write_varint(Self::CLIENTBOUND_ID);
        update_rot_packet.extend(write_varint(self.payload.e_id));
        update_rot_packet.extend(self.payload.yaw.to_be_bytes());
        update_rot_packet.extend(self.payload.pitch.to_be_bytes());
        update_rot_packet.push(self.payload.on_ground as u8);
        self.write_packet(update_rot_packet).await;
    }
}
