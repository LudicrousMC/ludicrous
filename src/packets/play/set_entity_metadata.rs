use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

/**
    Payload for the `Set Entity Metadata` Packet

    # Fields
        * e_id: Id of the entity
        * metadata: Byte array of the entity metadata
*/
pub struct SetEntityMetadataPayload {
    pub e_id: i32,
    pub metadata: Vec<u8>,
}

/**
    Set Entity Metadata Packet
    Sets the entity metadata for a given entity on the client

    # Clientbound
        * id: `0x5C`
        * resource: `set_entity_data`

    # Serverbound
        * No relevant serverbound packet
*/
pub struct SetEntityMetadata {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    payload: SetEntityMetadataPayload,
}

impl SetEntityMetadata {
    pub fn new(
        player: Arc<AMutex<Player>>,
        socket_write: Arc<AMutex<OwnedWriteHalf>>,
        payload: SetEntityMetadataPayload,
    ) -> Self {
        SetEntityMetadata {
            player,
            socket_write,
            payload,
        }
    }
}

#[async_trait::async_trait]
impl Packet for SetEntityMetadata {
    const CLIENTBOUND_ID: i32 = 0x5C;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut entity_metadata_packet = write_varint(Self::CLIENTBOUND_ID);
        entity_metadata_packet.extend(write_varint(self.payload.e_id)); // entity id
        entity_metadata_packet.extend(&self.payload.metadata); // entity metadata
        self.format_packet(&mut entity_metadata_packet);
        self.encrypt_packet(&mut entity_metadata_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&entity_metadata_packet)
            .await
            .unwrap();
    }
}
