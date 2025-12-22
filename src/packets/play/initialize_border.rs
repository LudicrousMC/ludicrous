use super::super::{write_varint, Packet, PacketStatic, Player, PlayerWriteConn};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex as AMutex;

pub struct InitializeBorder;

impl InitializeBorder {
    pub fn new() -> Self {
        InitializeBorder
    }
}

impl PacketStatic for InitializeBorder {
    const CLIENTBOUND_ID: i32 = 0x25;
}

#[async_trait::async_trait]
impl Packet for InitializeBorder {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut init_border_packet = write_varint(Self::CLIENTBOUND_ID);
        init_border_packet.extend(0u64.to_be_bytes()); // X
        init_border_packet.extend(0u64.to_be_bytes()); // Y
        init_border_packet.extend(4723321869241942016u64.to_be_bytes()); // old diameter
        init_border_packet.extend(4723321869241942016u64.to_be_bytes()); // new diameter
        init_border_packet.push(0x00); // border speed (varlong)
        init_border_packet.extend(write_varint(29999984)); // portal tel bound
        init_border_packet.extend(write_varint(5)); // warning blocks
        init_border_packet.extend(write_varint(15)); // warning time
        write_conn.write_packet(init_border_packet).await;
    }
}
