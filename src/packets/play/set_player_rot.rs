use super::super::{Packet, Player};
use std::sync::Arc;
use tokio::sync::Mutex as AMutex;

/**
    Set Player Rotation Packet
    Sets a player's rotation and broadcasts to nearby players

    # Serverbound
        * id: `0x1F`
        * resource: `move_player_rot`

    # Clientbound
        * See `Update Entity Rotation` (update_entity_rot.rs)
*/
pub struct SetPlayerRot<'a> {
    player: Arc<AMutex<Player>>,
    packet_data: &'a [u8],
}

impl<'a> SetPlayerRot<'a> {
    pub fn new(player: Arc<AMutex<Player>>, packet_data: &'a [u8]) -> Self {
        SetPlayerRot {
            player,
            packet_data,
        }
    }
}

#[async_trait::async_trait]
impl Packet for SetPlayerRot<'_> {
    const SERVERBOUND_ID: i32 = 0x1F;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn receive(&self) {
        let mut yaw = [0u8; 4];
        yaw.copy_from_slice(&self.packet_data[..4]);
        let yaw = f32::from_be_bytes(yaw);
        let mut pitch = [0u8; 4];
        pitch.copy_from_slice(&self.packet_data[4..8]);
        let pitch = f32::from_be_bytes(pitch);

        //self.player.lock().await.update_rotation(yaw, pitch).await;
    }
}
