use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

/**
    Set Container Content Packet
    Sets slots for a container

    # Clientbound
    * id: `0x12`
    * resource: `container_set_content`

    # Serverbound
        No relevant serverbound packet
*/
pub struct SetContainerContent {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
}

impl SetContainerContent {
    pub fn new(player: Arc<AMutex<Player>>, socket_write: Arc<AMutex<OwnedWriteHalf>>) -> Self {
        SetContainerContent {
            player,
            socket_write,
        }
    }
}

#[async_trait::async_trait]
impl Packet for SetContainerContent {
    const CLIENTBOUND_ID: i32 = 0x12;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut set_container_packet = write_varint(Self::CLIENTBOUND_ID);
        set_container_packet.extend(write_varint(0x00)); // inv window id
        set_container_packet.extend(write_varint(0x04)); // state id
        set_container_packet.extend(write_varint(46));
        let mut container_content = Vec::new();
        for i in 0..=46 {
            let count = self.player.lock().await.data.inventory.items[i].count;
            container_content.extend(write_varint(count.into()));
            if count > 0 {
                container_content.extend(write_varint(
                    self.player.lock().await.data.inventory.items[i].id as i32,
                ));
                container_content.extend([0x00, 0x00]);
            }
        }
        println!("{:?}", container_content.len());

        set_container_packet.extend(container_content);
        self.format_packet(&mut set_container_packet);
        self.encrypt_packet(&mut set_container_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&set_container_packet)
            .await
            .unwrap();
    }
}
