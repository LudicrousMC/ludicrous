use super::super::{write_string, write_varint, Packet, PacketStatic, PlayerWriteConn};

pub struct PlayerInfoUpdate;

impl PlayerInfoUpdate {
    pub fn new() -> Self {
        PlayerInfoUpdate
    }
}

impl PacketStatic for PlayerInfoUpdate {
    const CLIENTBOUND_ID: i32 = 0x3F;
}

#[async_trait::async_trait]
impl Packet for PlayerInfoUpdate {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut info_update_packet = write_varint(Self::CLIENTBOUND_ID);
        info_update_packet.push(0xFF); // actions
        info_update_packet.push(0x01); // player array len
        info_update_packet.extend(write_conn.data.clone().unwrap().uuid); // add player uuid
        info_update_packet.extend(write_string("Dev"));
        info_update_packet.push(0x00); // player signature false
        info_update_packet.push(0x00); // init chat optional = false
        info_update_packet.push(0x00); // gamemode 0
        info_update_packet.push(0x01); // set player listed
        info_update_packet.push(0x00); // latency
        info_update_packet.push(0x00); // optional display name
        info_update_packet.push(0x00); // list priority
        info_update_packet.push(0x01); // show player hat layer
        write_conn.write_packet(info_update_packet).await;
    }
}
