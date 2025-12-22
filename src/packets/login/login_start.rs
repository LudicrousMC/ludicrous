use super::super::{
    super::player::PlayerState, super::server::ServerData, Packet, PacketMode, PacketStatic,
    Player, PlayerReadConn, PlayerStream,
};
use sha1::Digest;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct LoginStart {
    server: Arc<ServerData>,
    player_data: Option<Arc<Player>>,
    pub high_channel: Option<(Sender<Box<dyn Packet>>, Receiver<Box<dyn Packet>>)>,
    pub low_channel: Option<(Sender<Box<dyn Packet>>, Receiver<Box<dyn Packet>>)>,
}

impl LoginStart {
    pub fn new(server: Arc<ServerData>) -> Self {
        LoginStart {
            server,
            player_data: None,
            high_channel: None,
            low_channel: None,
        }
    }
}

impl PacketStatic for LoginStart {
    const SERVERBOUND_ID: i32 = 0x00;
    const PACKET_MODE: PacketMode = PacketMode::Receive;
}

#[async_trait::async_trait]
impl Packet for LoginStart {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn handle(&mut self, conn: &mut PlayerStream) {
        self.receive(&mut conn.read).await;
        conn.read.data = self.player_data.clone();
        conn.write.data = self.player_data.take();
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        let _login_packet_len = read_conn.read_varint().await;
        let _login_packet_id = read_conn.read_varint().await;
        let mut username_buf = vec![0u8; read_conn.read_varint().await as usize];
        read_conn
            .socket_read
            .read_exact(&mut username_buf)
            .await
            .unwrap();
        let username = String::from_utf8_lossy(&username_buf).to_string();

        let mut uuid_buf = [0u8; 16];
        read_conn
            .socket_read
            .read_exact(&mut uuid_buf)
            .await
            .unwrap();

        let uuid = uuid_buf
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join("");
        let mut offline_uuid_hash =
            sha1::Sha1::digest(format!("OfflinePlayer:{}", username).as_bytes());
        let mut offline_uuid = [0u8; 16];
        offline_uuid.copy_from_slice(&offline_uuid_hash[..16]);
        offline_uuid[6] = (offline_uuid[6] & 0x0F) | 0x30;
        offline_uuid[8] = (offline_uuid[8] & 0x3F) | 0x80;
        let (tx, rx) = tokio::sync::mpsc::channel(4096);
        let (low_tx, low_rx) = tokio::sync::mpsc::channel(4096);
        let player_data = Player::new(
            PlayerState::Login,
            username.clone(),
            uuid_buf,
            self.server.clone(),
            tx.downgrade(),
            low_tx.downgrade(),
        );
        self.player_data = Some(player_data);
        self.high_channel = Some((tx, rx));
        self.low_channel = Some((low_tx, low_rx));
        /* *self.player.lock().unwrap() = Some(Player::new(
            PlayerState::Login,
            username,
            offline_uuid,
            self.server.clone(),
            self.socket_read.clone(),
            self.socket_write.clone(),
        ));*/
    }
}
