use super::super::{
    super::PlayerState, write_string, write_varint, Packet, PacketMode, PacketStatic,
    PlayerReadConn, PlayerWriteConn,
};

pub struct LoginSuccess;

impl LoginSuccess {
    pub fn new() -> Self {
        LoginSuccess
    }
}

impl PacketStatic for LoginSuccess {
    const CLIENTBOUND_ID: i32 = 0x02;
    const SERVERBOUND_ID: i32 = 0x03;
    const PACKET_MODE: PacketMode = PacketMode::SendThenReceive;
}

#[async_trait::async_trait]
impl Packet for LoginSuccess {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        // Login Success Request Packet (S -> C)
        let mut login_success_packet = write_varint(Self::CLIENTBOUND_ID);
        let player_data = write_conn.data.as_ref().unwrap().clone();
        login_success_packet.extend(player_data.uuid); // Player UUID
        login_success_packet.extend(write_string(&player_data.username)); // Username string
        login_success_packet.extend(write_varint(0)); // Empty array length
        write_conn.write_packet(login_success_packet).await;
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        let _login_acknowledged_packet = read_conn.decrypt_data().await.unwrap();
        read_conn.data.clone().unwrap().state.store(
            PlayerState::Configuration as u8,
            std::sync::atomic::Ordering::Relaxed,
        );
    }
}
