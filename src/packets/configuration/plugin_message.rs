use super::super::{
    write_string, write_varint, Packet, PacketStatic, PlayerReadConn, PlayerWriteConn,
};
use tokio::io::AsyncWriteExt;

pub struct PluginMessage;

impl PluginMessage {
    pub fn new() -> Self {
        PluginMessage
    }
}

impl PacketStatic for PluginMessage {
    const CLIENTBOUND_ID: i32 = 0x01;
    const SERVERBOUND_ID: i32 = 0x02;
}

#[async_trait::async_trait]
impl Packet for PluginMessage {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut plugin_message_packet = write_varint(Self::CLIENTBOUND_ID);
        plugin_message_packet.extend(write_string("minecraft:brand")); // plugin channel
        plugin_message_packet.extend(write_string(
            "\u{00A7}6\u{00A7}l馬鹿げてる ludicrous\u{00A7}r",
        )); // byte arr data
        write_conn.write_packet(plugin_message_packet).await;
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        // Empty
    }
}
