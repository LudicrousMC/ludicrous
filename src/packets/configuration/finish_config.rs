use super::super::{
    super::PlayerState, write_varint, Packet, PacketStatic, PlayerReadConn, PlayerWriteConn,
};

pub struct FinishConfig;

impl FinishConfig {
    pub fn new() -> Self {
        FinishConfig
    }
}

impl PacketStatic for FinishConfig {
    const CLIENTBOUND_ID: i32 = 0x03;
}

#[async_trait::async_trait]
impl Packet for FinishConfig {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let finish_config_packet = write_varint(Self::CLIENTBOUND_ID);
        write_conn.write_packet(finish_config_packet).await;
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        read_conn
            .data
            .clone()
            .unwrap()
            .state
            .store(PlayerState::Play as u8, std::sync::atomic::Ordering::SeqCst);
    }
}
