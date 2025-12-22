use super::super::{
    super::server::ServerData, read_varint_from_vec, write_string, write_varint, Packet,
    PacketStatic, Player, PlayerWriteConn,
};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex as AMutex;

pub struct Login {
    server: Arc<ServerData>,
}

impl Login {
    pub fn new(server: Arc<ServerData>) -> Self {
        Login { server }
    }
}

impl PacketStatic for Login {
    const CLIENTBOUND_ID: i32 = 0x2B;
}

#[async_trait::async_trait]
impl Packet for Login {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut login_packet = write_varint(Self::CLIENTBOUND_ID);
        let player_data = write_conn.data.clone().unwrap().clone();
        login_packet.extend(player_data.id.to_be_bytes()); // player id
        login_packet.push(0x00); // is hardcore
        login_packet.extend(write_varint(0x01)); // dimensions
        login_packet.extend(write_string("minecraft:overworld"));
        login_packet.extend(write_varint(0x02)); // max players
        login_packet.extend(write_varint(self.server.config.view_distance as i32)); // view distance
        login_packet.extend(write_varint(self.server.config.simulation_distance as i32)); // sim distance
        login_packet.push(0x00); // reduced debug
        login_packet.push(0x01); // respawn screen
        login_packet.extend(write_varint(0x00)); // do limited crafting
        login_packet.extend(write_varint(0x00)); // dimension type
                                                 //login_packet.extend(write_varint("overworld".len() as i32).await); // dim name len
        login_packet.extend(write_string("minecraft:overworld")); // dim name
        login_packet.extend(0i64.to_be_bytes()); // hashed seed
        login_packet.push(0x01); // game mode
        login_packet.push(0x01); // previous game mode
        login_packet.push(0x00); // is debug
        login_packet.push(0x01); // is flat
        login_packet.push(0x00); // has death location
        login_packet.extend(write_varint(0x00)); // Portal cooldown
        login_packet.extend(write_varint(0x00)); // sea level
        login_packet.push(0x00); // secure chat
        write_conn.write_packet(login_packet).await;
    }
}
