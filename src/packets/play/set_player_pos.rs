use super::super::{super::LudiChunkLoader, Packet, PacketStatic, Player, PlayerReadConn};

/**
    Set Player Position Packet
    Sets a player's position and broadcasts to nearby players

    # Serverbound
        * id: `0x1D`
        * resource: `move_player_pos`

    # Clientbound
        * See `Update Entity Position` (update_entity_pos.rs)
*/
pub struct SetPlayerPos<'a> {
    packet_data: &'a [u8],
}

impl<'a> SetPlayerPos<'a> {
    pub fn new(packet_data: &'a [u8]) -> Self {
        SetPlayerPos { packet_data }
    }
}

impl PacketStatic for SetPlayerPos<'_> {
    const SERVERBOUND_ID: i32 = 0x1D;
}

#[async_trait::async_trait]
impl Packet for SetPlayerPos<'_> {
    async fn receive(&mut self, read_conn: &mut PlayerReadConn) {
        let mut x = [0u8; 8];
        x.copy_from_slice(&self.packet_data[..8]);
        let x = f64::from_be_bytes(x);
        let mut y = [0u8; 8];
        y.copy_from_slice(&self.packet_data[8..16]);
        let y = f64::from_be_bytes(y);
        let mut z = [0u8; 8];
        z.copy_from_slice(&self.packet_data[16..24]);
        let z = f64::from_be_bytes(z);
        let old_player_pos = read_conn.data.clone().unwrap().get_position();
        read_conn
            .data
            .clone()
            .unwrap()
            .update_position(x, y, z)
            .await;
        let new_player_pos = read_conn.data.clone().unwrap().get_position();
        let old_center_chunk = LudiChunkLoader::pos_to_chunk(old_player_pos.0, old_player_pos.2);
        let new_center_chunk = LudiChunkLoader::pos_to_chunk(new_player_pos.0, new_player_pos.2);
        read_conn
            .data
            .clone()
            .unwrap()
            .server
            .load_chunks(
                new_center_chunk,
                old_center_chunk,
                read_conn.data.clone().unwrap(),
            )
            .await;
        /*tokio::spawn({
            let player = self.player.clone();
            async move {
                player.lock().await.update_position(x, y, z).await;
            }
        });*/
    }
}
