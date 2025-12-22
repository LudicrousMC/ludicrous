use super::super::{
    super::PlayerState, read_varint_from_vec, write_varint, Packet, PacketMode, PacketStatic,
    Player, PlayerReadConn, PlayerWriteConn,
};

/**
    Synchronize Player Position Packet

    # Clientbound
        * id: `0x41`
        * resource: `player_position`

    # Serverbound
        * id: `0x00`
        * resource: `accept_teleportation`
*/
pub struct SynchronizePlayerPos;

impl SynchronizePlayerPos {
    pub fn new() -> Self {
        SynchronizePlayerPos
    }
}

impl PacketStatic for SynchronizePlayerPos {
    const CLIENTBOUND_ID: i32 = 0x41;
    const SERVERBOUND_ID: i32 = 0x00;
}

#[async_trait::async_trait]
impl Packet for SynchronizePlayerPos {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let player = write_conn.data.clone().unwrap().clone();
        let pos = player.get_position();
        let mut synchronize_pos_packet = write_varint(Self::CLIENTBOUND_ID);
        synchronize_pos_packet.extend(write_varint(
            player
                .counters
                .teleport_id
                .load(std::sync::atomic::Ordering::Relaxed) as i32,
        )); // teleport id
        synchronize_pos_packet.extend(pos.0.to_be_bytes());
        synchronize_pos_packet.extend(pos.1.to_be_bytes());
        synchronize_pos_packet.extend(pos.2.to_be_bytes());

        //synchronize_pos_packet.extend(22000000f64.to_be_bytes()); // x pos
        //synchronize_pos_packet.extend(160f64.to_be_bytes()); // y pos
        //synchronize_pos_packet.extend((-25000000f64).to_be_bytes()); // z pos
        synchronize_pos_packet.extend(0i64.to_be_bytes()); // x vel
        synchronize_pos_packet.extend(0i64.to_be_bytes()); // y vel
        synchronize_pos_packet.extend(0i64.to_be_bytes()); // z vel
        synchronize_pos_packet.extend(0i32.to_be_bytes()); // yaw
        synchronize_pos_packet.extend(0i32.to_be_bytes()); // pitch
        synchronize_pos_packet.extend(0i32.to_be_bytes()); // teleport flags
        write_conn.write_packet(synchronize_pos_packet).await;
    }

    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        let mut confirm_teleport_packet = read_conn.decrypt_packet().await.unwrap();
        //let _confirm_teleport_packet_id = read_varint_from_vec(&mut confirm_teleport_packet);
        read_conn
            .data
            .clone()
            .unwrap()
            .counters
            .teleport_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}
