use super::super::{write_varint, Packet, PacketStatic, Player, PlayerWriteConn};

pub struct PlayerAbilities {
    flags: u8,
    fly_speed: f32,
    fov_mod: f32,
}

impl PlayerAbilities {
    pub fn new(flags: u8, fly_speed: f32, fov_mod: f32) -> Self {
        PlayerAbilities {
            flags,
            fly_speed,
            fov_mod,
        }
    }
}

impl PacketStatic for PlayerAbilities {
    const CLIENTBOUND_ID: i32 = 0x39;
}

#[async_trait::async_trait]
impl Packet for PlayerAbilities {
    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut abilities_packet = write_varint(Self::CLIENTBOUND_ID);
        abilities_packet.push(self.flags);
        abilities_packet.extend(self.fly_speed.to_be_bytes());
        abilities_packet.extend(self.fov_mod.to_be_bytes());
        write_conn.write_packet(abilities_packet).await;
    }
}
