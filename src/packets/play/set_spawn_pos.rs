use super::super::{write_varint, Packet, Player, super::server::ServerData};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct SetSpawnPos {
    player: Arc<AMutex<Player>>,
    socket_write: Arc<AMutex<OwnedWriteHalf>>,
    server: Arc<ServerData>,
}

impl SetSpawnPos {
    pub fn new(player: Arc<AMutex<Player>>, socket_write: Arc<AMutex<OwnedWriteHalf>>, server: Arc<ServerData>) -> Self {
        SetSpawnPos {
            player,
            socket_write,
            server,
        }
    }
}

#[async_trait::async_trait]
impl Packet for SetSpawnPos {
    const CLIENTBOUND_ID: i32 = 0x5A;

    fn player(&self) -> Option<Arc<AMutex<Player>>> {
        Some(self.player.clone())
    }

    async fn send(&self) {
        let server_spawn_x = self.server.level.spawn_x as i64;
        let server_spawn_y = self.server.level.spawn_y as i64;
        let server_spawn_z = self.server.level.spawn_z as i64;
        let spawn_pos = ((server_spawn_x & 0x3FFFFFF) << 38)
            | ((server_spawn_z & 0x3FFFFFF) << 12)
            | (server_spawn_y & 0xFFF);
        let server_spawn_rot = self.server.level.spawn_angle;
        let mut spawn_pos_packet = write_varint(Self::CLIENTBOUND_ID);
        spawn_pos_packet.extend(spawn_pos.to_be_bytes()); // spawn location
        spawn_pos_packet.extend(server_spawn_rot.to_be_bytes()); // spawn rotation
        self.format_packet(&mut spawn_pos_packet);
        self.encrypt_packet(&mut spawn_pos_packet).await;
        self.socket_write
            .lock()
            .await
            .write_all(&spawn_pos_packet)
            .await
            .unwrap();
    }
}
