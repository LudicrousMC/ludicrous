use super::super::{write_varint, Packet, PacketMode, PacketStatic, PlayerWriteConn};

pub struct SetCompression {
    threshold: i32,
}

impl SetCompression {
    pub fn new(threshold: i32) -> Self {
        Self { threshold }
    }
}

impl PacketStatic for SetCompression {
    const CLIENTBOUND_ID: i32 = 0x03;
    const PACKET_MODE: PacketMode = PacketMode::Send;
}

#[async_trait::async_trait]
impl Packet for SetCompression {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut set_compression_pkt = write_varint(Self::CLIENTBOUND_ID);
        set_compression_pkt.extend(write_varint(self.threshold));
        write_conn.write_packet(set_compression_pkt).await;
        write_conn
            .data
            .clone()
            .unwrap()
            .compression_enabled
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
