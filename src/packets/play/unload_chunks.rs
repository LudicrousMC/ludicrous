use super::super::{
    super::server::LudiChunkLoader, write_varint, Packet, PacketStatic, Player, PlayerWriteConn,
};

pub struct UnloadChunks {
    chunk_unloads: Vec<u64>,
}

impl UnloadChunks {
    pub fn new(chunk_unloads: Vec<u64>) -> Self {
        UnloadChunks { chunk_unloads }
    }
}

impl PacketStatic for UnloadChunks {
    const CLIENTBOUND_ID: i32 = 0x21;
}

#[async_trait::async_trait]
impl Packet for UnloadChunks {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        for chunk in &self.chunk_unloads {
            let chunk_coords = LudiChunkLoader::unpack_coords(*chunk);
            let mut chunk_unload_packet = write_varint(Self::CLIENTBOUND_ID);
            chunk_unload_packet.extend(chunk_coords.1.to_be_bytes());
            chunk_unload_packet.extend(chunk_coords.0.to_be_bytes());
            write_conn.write_packet(chunk_unload_packet).await;
        }
    }
}
