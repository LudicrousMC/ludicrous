use super::super::{
    super::server::chunk_system::BIOMES, write_string, write_varint, Packet, PacketMode,
    PacketStatic, PlayerWriteConn,
};
use tokio::io::AsyncWriteExt;

pub struct RegistryData;

impl RegistryData {
    pub fn new() -> Self {
        RegistryData
    }
}

impl PacketStatic for RegistryData {
    const CLIENTBOUND_ID: i32 = 0x07;
    const PACKET_MODE: PacketMode = PacketMode::Send;
}

#[async_trait::async_trait]
impl Packet for RegistryData {
    fn mode(&self) -> PacketMode {
        Self::PACKET_MODE
    }

    async fn send(&mut self, write_conn: &mut PlayerWriteConn) {
        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("dimension_type")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("overworld")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;
        /*self.format_packet(&mut registry_packet);
        self.encrypt_packet(&mut registry_packet);
        self.player
            .socket_write
            .write_all(&registry_packet)
            .await
            .unwrap();*/

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("wolf_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("ashen")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("wolf_sound_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("cute")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("pig_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("warm")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("frog_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("warm")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("cat_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("siamese")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("cow_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("warm")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("chicken_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("warm")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("chat_type")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("chat")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("trim_material")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("amethyst")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("painting_variant")); // reg id
        registry_packet.extend(write_varint(1)); // entries length
        registry_packet.extend(write_string("bomb")); // entry id
        registry_packet.push(0x00); // has data bool
        write_conn.write_packet(registry_packet).await;

        let damage_types = vec![
            "arrow",
            "bad_respawn_point",
            "cactus",
            "campfire",
            "cramming",
            "dragon_breath",
            "drown",
            "dry_out",
            "ender_pearl",
            "explosion",
            "fall",
            "falling_anvil",
            "falling_block",
            "falling_stalactite",
            "fireball",
            "fireworks",
            "fly_into_wall",
            "freeze",
            "generic",
            "generic_kill",
            "hot_floor",
            "in_fire",
            "in_wall",
            "indirect_magic",
            "lava",
            "lightning_bolt",
            "mace_smash",
            "magic",
            "mob_attack",
            "mob_attack_no_aggro",
            "mob_projectile",
            "on_fire",
            "out_of_world",
            "outside_border",
            "player_attack",
            "player_explosion",
            "sonic_boom",
            "spit",
            "stalagmite",
            "starve",
            "sting",
            "sweet_berry_bush",
            "thorns",
            "thrown",
            "trident",
            "unattributed_fireball",
            "wind_charge",
            "wither",
            "wither_skull",
        ];
        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("damage_type")); // reg id
        registry_packet.extend(write_varint(damage_types.len() as i32)); // entries length
        for d_type in damage_types.iter() {
            registry_packet.extend(write_string(d_type)); // entry id
            registry_packet.push(0x00); // has data bool
        }
        write_conn.write_packet(registry_packet).await;

        let mut registry_packet = write_varint(Self::CLIENTBOUND_ID);
        registry_packet.extend(write_string("worldgen/biome")); // reg id
        registry_packet.extend(write_varint(BIOMES.len() as i32)); // entries length
        for biome in BIOMES.iter() {
            registry_packet.extend(write_string(biome)); // entry id
            registry_packet.push(0x00); // has data bool
        }
        write_conn.write_packet(registry_packet).await;
    }
}
