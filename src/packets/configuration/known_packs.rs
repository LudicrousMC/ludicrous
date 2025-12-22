use super::super::{
    read_varint_from_vec, write_string, write_varint, Packet, PacketStatic, PlayerReadConn,
    PlayerWriteConn,
};
use crate::{MC_PROTOCOL, MC_VERSION};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct KnownPacks;

impl KnownPacks {
    pub fn new() -> Self {
        KnownPacks
    }
}

impl PacketStatic for KnownPacks {
    const CLIENTBOUND_ID: i32 = 0x0E;
    const SERVERBOUND_ID: i32 = 0x07;
}

#[async_trait::async_trait]
impl Packet for KnownPacks {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut known_packs_packet = write_varint(Self::CLIENTBOUND_ID);
        known_packs_packet.extend(write_varint(1)); // array size
        known_packs_packet.extend(write_string("minecraft")); // namespace
        known_packs_packet.extend(write_string("core")); // pack id
        known_packs_packet.extend(write_string(MC_VERSION)); // pack version
        write_conn.write_packet(known_packs_packet).await;
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        //let packet = read_conn.decrypt_packet().await.unwrap();
        //println!("known packs: {:02X?}", packet);
        //println!("test: {:02X?}", self.decrypt_packet().await);
        /*println!(
            "known packs?: {:02X?}",
            decrypt_packet(self.player.clone()).await
        );*/
    }
}
