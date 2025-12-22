use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

/**
    Remove Entities Packet
    Removes entites from the client by entity id

    # Clientbound
        * id: `0x46`
        * resource: `remove_entities`

    # Serverbound
        * See `Spawn Entity` packet. (spawn_entity.rs)
*/
pub struct RemoveEntities {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    entities: Vec<u8>,
}

impl RemoveEntities {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        entities: Vec<u8>,
    ) -> Self {
        RemoveEntities {
            player,
            socket_write,
            entities,
        }
    }
}

#[async_trait::async_trait]
impl Packet for RemoveEntities {
    const CLIENTBOUND_ID: i32 = 0x46;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut remove_entities_packet = write_varint(Self::CLIENTBOUND_ID);
        let mut entity_ids = self.entities.clone();
        self.format_packet(&mut entity_ids);
        remove_entities_packet.extend(entity_ids);
        self.format_packet(&mut remove_entities_packet);
        self.encrypt_packet(&mut remove_entities_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&remove_entities_packet)
            .await
            .unwrap();
    }
}
