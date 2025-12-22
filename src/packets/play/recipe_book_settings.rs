use super::super::{write_varint, Packet, Player};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct RecipeBookSettings {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
}

impl RecipeBookSettings {
    pub fn new(player: Arc<AMutex<Player>>, socket_write: Arc<AMutex<OwnedWriteHalf>>) -> Self {
        RecipeBookSettings {
            player,
            socket_write,
        }
    }
}

#[async_trait::async_trait]
impl Packet for RecipeBookSettings {
    const CLIENTBOUND_ID: i32 = 0x45;
    const SERVERBOUND_ID: i32 = 0x2C;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let mut rb_settings_packet = write_varint(Self::CLIENTBOUND_ID);
        rb_settings_packet.push(0x00); // r book open
        rb_settings_packet.push(0x00); // r book filter
        rb_settings_packet.push(0x00); // r smelt book open
        rb_settings_packet.push(0x00); // r smelt book filter
        rb_settings_packet.push(0x00); // r blast book open
        rb_settings_packet.push(0x00); // r blast book filter
        rb_settings_packet.push(0x00); // r smoker book open
        rb_settings_packet.push(0x00); // r smoker book filter
        self.format_packet(&mut rb_settings_packet);
        self.encrypt_packet(&mut rb_settings_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&rb_settings_packet)
            .await
            .unwrap();
    }
}
