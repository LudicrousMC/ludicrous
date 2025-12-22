use super::super::{write_varint, Packet, PacketStatic, PlayerWriteConn};
use tokio::io::AsyncWriteExt;

pub struct UpdateTags;

impl UpdateTags {
    pub fn new() -> Self {
        UpdateTags
    }
}

impl PacketStatic for UpdateTags {
    const CLIENTBOUND_ID: i32 = 0x0D;
}

#[async_trait::async_trait]
impl Packet for UpdateTags {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut vanilla_tags_packet = write_varint(Self::CLIENTBOUND_ID);
        vanilla_tags_packet.extend(
            std::fs::read("assets/vanilla-update-tags-payload.bin")
                .expect("assets/vanilla-update-tags-payload.bin file"),
        );
        write_conn.write_packet(vanilla_tags_packet).await;
    }
}
