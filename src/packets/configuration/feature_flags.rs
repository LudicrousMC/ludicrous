use super::super::{write_string, write_varint, Packet, PacketStatic, PlayerWriteConn};
use tokio::io::AsyncWriteExt;

pub struct FeatureFlags;

impl FeatureFlags {
    pub fn new() -> Self {
        FeatureFlags
    }
}

impl PacketStatic for FeatureFlags {
    const CLIENTBOUND_ID: i32 = 0x0C;
}

#[async_trait::async_trait]
impl Packet for FeatureFlags {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut feature_flags_packet = write_varint(Self::CLIENTBOUND_ID);
        feature_flags_packet.extend(write_varint(1)); // arr len
        feature_flags_packet.extend(write_string("minecraft:vanilla")); // feature flag
        write_conn.write_packet(feature_flags_packet).await;
    }
}
