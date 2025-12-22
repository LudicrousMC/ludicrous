use super::super::{
    super::server::ServerData, write_varint, Packet, PacketStatic, Player, PlayerWriteConn,
};
use std::sync::Arc;

/**
    Keep Alive Packet

    # Clientbound
    * id: `0x26`
    * resource: `keep_alive`

    # Serverbound
    * id: `0x1A`
    * resource: `keep_alive`
*/
pub struct KeepAlive {
    server: Arc<ServerData>,
}

impl KeepAlive {
    pub fn new(server: Arc<ServerData>) -> Self {
        KeepAlive { server }
    }
}

impl PacketStatic for KeepAlive {
    const CLIENTBOUND_ID: i32 = 0x26;
    const SERVERBOUND_ID: i32 = 0x1A;
}

#[async_trait::async_trait]
impl Packet for KeepAlive {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let time_elapsed: u64 = self
            .server
            .start_time
            .elapsed()
            .as_millis()
            .try_into()
            .expect("u64 type");
        let mut keep_alive_packet = write_varint(Self::CLIENTBOUND_ID);
        keep_alive_packet.extend(time_elapsed.to_be_bytes());
        write_conn.write_packet(keep_alive_packet).await;
    }
}
