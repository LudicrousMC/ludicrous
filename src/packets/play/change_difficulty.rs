use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct ChangeDifficulty {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
}

impl ChangeDifficulty {
    pub fn new(player: Arc<AMutex<Player>>, socket_write: Arc<AMutex<OwnedWriteHalf>>) -> Self {
        ChangeDifficulty {
            player,
            socket_write,
        }
    }
}

#[async_trait::async_trait]
impl Packet for ChangeDifficulty {
    const CLIENTBOUND_ID: i32 = 0x0A;
    const SERVERBOUND_ID: i32 = 0x03;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut difficulty_packet = write_varint(Self::CLIENTBOUND_ID);
        difficulty_packet.push(0x00); // peaceful difficulty
        difficulty_packet.push(0x00); // difficulty not locked
        self.format_packet(&mut difficulty_packet);
        self.encrypt_packet(&mut difficulty_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&difficulty_packet)
            .await
            .unwrap();
    }
}
