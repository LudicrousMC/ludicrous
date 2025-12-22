use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

/**
    Payload for the `Set Entity Velocity` Packet

    # Fields
        * e_id: Id of the entity
        * vel_x: X velocity of the entity
        * vel_y: Y velocity of the entity
        * vel_z: Z velocity of the entity
*/
pub struct SetEntityVelPayload {
    pub e_id: i32,
    pub vel_x: i16,
    pub vel_y: i16,
    pub vel_z: i16,
}

/**
    Set Entity Velocity Packet
    Sets a entity's velocity for a client

    # Clientbound
        * id: `0x5E`
        * resource: `set_entity_motion`

    # Serverbound
        * No relevant serverbound packet
*/
pub struct SetEntityVel {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    payload: SetEntityVelPayload,
}

impl SetEntityVel {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        payload: SetEntityVelPayload,
    ) -> Self {
        SetEntityVel {
            player,
            socket_write,
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Packet for SetEntityVel {
    const CLIENTBOUND_ID: i32 = 0x5E;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut set_vel_packet = write_varint(Self::CLIENTBOUND_ID);
        set_vel_packet.extend(write_varint(self.payload.e_id));
        set_vel_packet.extend(self.payload.vel_x.to_be_bytes());
        set_vel_packet.extend(self.payload.vel_y.to_be_bytes());
        set_vel_packet.extend(self.payload.vel_z.to_be_bytes());
        self.format_packet(&mut set_vel_packet);
        self.encrypt_packet(&mut set_vel_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&set_vel_packet)
            .await
            .unwrap();
        println!("entity vel {}", self.player.lock().await.uuid[15]);
    }
}
