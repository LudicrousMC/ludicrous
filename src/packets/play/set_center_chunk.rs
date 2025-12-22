use super::super::{write_varint, Packet, PacketStatic, Player, PlayerWriteConn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

/**
    Set Center Chunk Packet
        Tells the client what chunk to load other chunks around

    # Clientbound
        * id: `0x57`
        * resource: `set_chunk_cache_center`

    # Serverbound
        * No relevant serverbound packet
*/
pub struct SetCenterChunk {
    center_chunk: (i32, i32),
}

impl SetCenterChunk {
    pub fn new(center_chunk: (i32, i32)) -> Self {
        SetCenterChunk { center_chunk }
    }
}

impl PacketStatic for SetCenterChunk {
    const CLIENTBOUND_ID: i32 = 0x57;
}

#[async_trait::async_trait]
impl Packet for SetCenterChunk {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut center_chunk_packet = write_varint(Self::CLIENTBOUND_ID);
        center_chunk_packet.extend(write_varint(self.center_chunk.0));
        center_chunk_packet.extend(write_varint(self.center_chunk.1));
        write_conn.write_packet(center_chunk_packet).await;
    }
}
