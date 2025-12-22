use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct BundleDelimiter {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
}

impl BundleDelimiter {
    pub fn new(player: Arc<AMutex<Player>>, socket_write: Arc<AMutex<OwnedWriteHalf>>) -> Self {
        BundleDelimiter {
            player,
            socket_write,
        }
    }
}

#[async_trait::async_trait]
impl Packet for BundleDelimiter {
    const CLIENTBOUND_ID: i32 = 0x00;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut bundle_packet = write_varint(Self::CLIENTBOUND_ID);
        self.format_packet(&mut bundle_packet);
        self.encrypt_packet(&mut bundle_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&bundle_packet)
            .await
            .unwrap();
        println!("bundle delim {}", self.player.lock().await.uuid[15]);
    }
}
