mod containers;
use super::entities::Entity;
use super::server::LudiChunkLoader;
use super::server::{ServerData, ServerMappings};
use super::Packet;
use containers::PlayerInventory;
use openssl::symm::Crypter;
use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{
    atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicU8, Ordering},
    Arc, RwLock,
};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::WeakSender;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerState {
    HandShake,
    Login,
    Configuration,
    Play,
}

impl PlayerState {
    pub fn from_u8(value: u8) -> Option<Self> {
        let state = match value {
            0 => Self::HandShake,
            1 => Self::Login,
            2 => Self::Configuration,
            3 => Self::Play,
            _ => return None,
        };
        Some(state)
    }
}

pub struct PlayerReadConn {
    pub data: Option<Arc<Player>>,
    pub socket_read: OwnedReadHalf,
    pub decryptor: Option<Crypter>,
    /// Intended for reusing space to avoid allocating new vec for each decryption
    pub encrypted_buf: Vec<u8>,
    /// Intended for saving previous excess stream data to avoid data loss
    pub decrypted_buf: Vec<u8>,
    pub decrypted_data: Vec<u8>,
}

pub struct PlayerWriteConn {
    pub data: Option<Arc<Player>>,
    pub socket_write: OwnedWriteHalf,
    pub encryptor: Option<Crypter>,
    /// Intended for reusing space to avoid allocating new vec for each encryption
    pub encrypt_buf: Vec<u8>,
}

pub struct PlayerStream {
    pub read: PlayerReadConn,
    pub write: PlayerWriteConn,
}

impl PlayerStream {
    pub fn new(socket: TcpStream) -> PlayerStream {
        let (socket_read, socket_write) = socket.into_split();
        PlayerStream {
            read: PlayerReadConn::new(socket_read),
            write: PlayerWriteConn::new(socket_write),
        }
    }

    pub fn split(self) -> (PlayerReadConn, PlayerWriteConn) {
        (self.read, self.write)
    }

    pub fn from(read: PlayerReadConn, write: PlayerWriteConn) -> PlayerStream {
        PlayerStream { read, write }
    }
}

impl PlayerReadConn {
    pub fn new(socket_read: OwnedReadHalf) -> PlayerReadConn {
        const BUF_LEN: usize = 1440;
        PlayerReadConn {
            socket_read,
            data: None,
            decryptor: None,
            encrypted_buf: vec![0u8; BUF_LEN],
            decrypted_buf: vec![0u8; BUF_LEN],
            decrypted_data: vec![],
        }
    }
}

impl PlayerWriteConn {
    pub fn new(socket_write: OwnedWriteHalf) -> PlayerWriteConn {
        PlayerWriteConn {
            socket_write,
            data: None,
            encryptor: None,
            encrypt_buf: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct PlayerCounters {
    pub entity_id: i32,
    pub teleport_id: AtomicU32,
}

impl Default for PlayerCounters {
    fn default() -> Self {
        Self {
            entity_id: 1,
            teleport_id: 0.into(),
        }
    }
}

#[derive(Debug)]
pub struct PlayerData {
    pub inventory: PlayerInventory,
    /// f64's represented as atomic u64
    pub pos: [AtomicU64; 3],
    pub rotation: [f32; 2],
    pub respawn: Option<PlayerRespawn>,
}

// Mutex and RwLock can be optimized with UnsafeCell where reads are multi-thread and writes are
// single thread
pub struct Player {
    // PlayerState as atomic u8
    pub state: AtomicU8,
    pub username: String,
    pub id: i32,
    pub uuid: [u8; 16],
    pub outbound: WeakSender<Box<dyn Packet>>,
    pub low_priority_outbound: WeakSender<Box<dyn Packet>>,
    pub counters: PlayerCounters,
    pub data: PlayerData,
    pub server: Arc<ServerData>,
    pub chunk: (AtomicI32, AtomicI32),
    // Players within simulation distance that will have data broadcasted to them
    // Identified by entity id. eid -> Player
    pub nearby_players: RwLock<HashMap<i32, Arc<Player>>>,
    // entity id -> Entity
    pub entities: HashMap<i32, Entity>,
    pub compression_enabled: AtomicBool,
}

impl std::fmt::Debug for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("player").finish()
    }
}

impl Player {
    pub fn new(
        state: PlayerState,
        username: String,
        uuid: [u8; 16],
        server: Arc<ServerData>,
        outbound1: WeakSender<Box<dyn Packet>>,
        outbound2: WeakSender<Box<dyn Packet>>,
    ) -> Arc<Self> {
        let level_name = server.config.level_name.clone();
        let nbt_file = std::fs::File::open(format!(
            "{}/playerdata/{}.dat",
            level_name,
            Self::get_uuid_string_from_bytes(uuid)
        ));
        let mut player_data = if let Ok(nbt) = nbt_file {
            let mut decoder = flate2::read::GzDecoder::new(nbt);
            let mut nbt_data = vec![];
            decoder.read_to_end(&mut nbt_data).unwrap();
            fastnbt::from_bytes::<PlayerNBT>(nbt_data.as_slice()).expect("Invalid player.dat NBT")
        } else {
            let mut new_player = PlayerNBT::default();
            new_player.rotation[0] = server.level.spawn_angle;
            new_player.pos[0] = server.level.spawn_x as f64;
            new_player.pos[1] = server.level.spawn_y as f64;
            new_player.pos[2] = server.level.spawn_z as f64;
            new_player
        };
        player_data.mappings = Some(&server.mappings);
        let data: PlayerData = player_data.into();
        let id = server.next_eid() as i32;
        let x = f64::from_bits(data.pos[0].load(Ordering::Relaxed));
        let z = f64::from_bits(data.pos[2].load(Ordering::Relaxed));
        let chunk = LudiChunkLoader::pos_to_chunk(x, z);
        Arc::new(Player {
            state: (state as u8).into(),
            username,
            id,
            uuid,
            outbound: outbound1,
            low_priority_outbound: outbound2,
            counters: Default::default(),
            data,
            server,
            chunk: (AtomicI32::new(chunk.0), AtomicI32::new(chunk.1)),
            nearby_players: RwLock::new(HashMap::new()),
            entities: HashMap::new(),
            compression_enabled: AtomicBool::new(false),
        })
    }

    pub fn get_uuid_string_from_bytes(uuid: [u8; 16]) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            uuid[0], uuid[1], uuid[2], uuid[3],
            uuid[4], uuid[5],
            uuid[6], uuid[7],
            uuid[8], uuid[9],
            uuid[10], uuid[11], uuid[12], uuid[13], uuid[14], uuid[15]
        )
    }

    pub fn get_uuid_string(&self) -> String {
        Self::get_uuid_string_from_bytes(self.uuid)
    }

    pub async fn update_position(&self, x: f64, y: f64, z: f64) {
        let delta_x = x - f64::from_bits(self.data.pos[0].load(Ordering::Relaxed));
        let delta_y = y - f64::from_bits(self.data.pos[1].load(Ordering::Relaxed));
        let delta_z = z - f64::from_bits(self.data.pos[2].load(Ordering::Relaxed));
        if delta_x == 0.0 && delta_y == 0.0 && delta_z == 0.0 {
            return;
        }
        self.data.pos[0].store(x.to_bits(), Ordering::Relaxed);
        self.data.pos[1].store(y.to_bits(), Ordering::Relaxed);
        self.data.pos[2].store(z.to_bits(), Ordering::Relaxed);
        //println!("{delta_x} {delta_y} {delta_z}");
        let x_in_bounds = (-8.0..8.0).contains(&delta_x);
        let y_in_bounds = (-8.0..8.0).contains(&delta_y);
        let z_in_bounds = (-8.0..8.0).contains(&delta_z);
        if x_in_bounds && y_in_bounds && z_in_bounds {
            // Event logic
        }
    }

    pub fn get_position(&self) -> (f64, f64, f64) {
        let x = f64::from_bits(self.data.pos[0].load(Ordering::Relaxed));
        let y = f64::from_bits(self.data.pos[1].load(Ordering::Relaxed));
        let z = f64::from_bits(self.data.pos[2].load(Ordering::Relaxed));
        (x, y, z)
    }

    pub fn f32_to_angle(angle: f32) -> i8 {
        let quadrant = ((angle * 2.0 * std::f32::consts::PI) / 360.0).sin();
        let new = if angle.is_sign_positive() {
            if quadrant.is_sign_positive() {
                angle % 180.0
            } else {
                (angle % 180.0) - 180.0
            }
        } else {
            if quadrant.is_sign_negative() {
                -((180.0 - angle) % 180.0)
            } else {
                180.0 + (angle % 180.0)
            }
        };
        (new * (256.0 / 360.0)) as i8
    }
}

/**
    Player NBT Data Struct
    This is an intermediary struct for accessing and writing NBT data to player.dat
*/
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct PlayerNBT<'a> {
    #[serde(skip_deserializing)]
    pub mappings: Option<&'a ServerMappings>,
    pub pos: [f64; 3],
    pub rotation: [f32; 2],
    pub inventory: Vec<ISlot<'a>>,
    #[serde(rename = "EnderItems")]
    pub ender_chest: Vec<ISlot<'a>>,
    #[serde(rename = "equipment")]
    pub equipment: Option<EquipmentNBT>,
    #[serde(rename = "respawn")]
    pub respawn: Option<PlayerRespawn>,
}

impl Into<PlayerData> for PlayerNBT<'_> {
    fn into(self) -> PlayerData {
        let mut pos = [const { AtomicU64::new(0) }; 3];
        (0..3).for_each(|i| pos[i] = AtomicU64::new(self.pos[i].to_bits()));
        PlayerData {
            inventory: PlayerInventory::from_nbt(&self, self.mappings.unwrap()),
            pos,
            rotation: self.rotation,
            respawn: self.respawn,
        }
    }
}

#[derive(Deserialize, Default, Debug)]
pub struct EquipmentNBT {
    pub offhand: Option<ISlot<'static>>,
    pub head: Option<ISlot<'static>>,
    pub chest: Option<ISlot<'static>>,
    pub legs: Option<ISlot<'static>>,
    pub feet: Option<ISlot<'static>>,
}

#[derive(Deserialize, Debug)]
pub struct ISlot<'a> {
    pub id: Cow<'a, str>,
    pub count: i8,
    #[serde(rename = "Slot")]
    pub slot: Option<u8>,
    pub tag: Option<fastnbt::Value>,
}

#[derive(Deserialize, Debug, Default)]
pub struct PlayerRespawn {
    pub angle: f32,
    pub pos: [i32; 3],
}
