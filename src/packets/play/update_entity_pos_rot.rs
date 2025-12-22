use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct UpdateEntityPosRotPayload {
    pub e_id: i32,
    pub delta_x: i16,
    pub delta_y: i16,
    pub delta_z: i16,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool,
}

pub struct UpdateEntityPosRot {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    payload: UpdateEntityPosRotPayload,
}

impl UpdateEntityPosRot {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        payload: UpdateEntityPosRotPayload,
    ) -> Self {
        UpdateEntityPosRot {
            player,
            socket_write,
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Packet for UpdateEntityPosRot {
    const CLIENTBOUND_ID: i32 = 0x2F;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    fn socket_write(&self) -> Option<Arc<AMutex<OwnedWriteHalf>>> {
        Some(self.socket_write.clone())
    }

    async fn send(&self) {
        let mut update_pos_rot_packet = write_varint(Self::CLIENTBOUND_ID);
        update_pos_rot_packet.extend(write_varint(self.payload.e_id));
        update_pos_rot_packet.extend(self.payload.delta_x.to_be_bytes());
        update_pos_rot_packet.extend(self.payload.delta_y.to_be_bytes());
        update_pos_rot_packet.extend(self.payload.delta_z.to_be_bytes());
        update_pos_rot_packet.extend(self.payload.yaw.to_be_bytes());
        update_pos_rot_packet.extend(self.payload.pitch.to_be_bytes());
        update_pos_rot_packet.push(self.payload.on_ground as u8);
        self.write_packet(update_pos_rot_packet).await;
    }
}
