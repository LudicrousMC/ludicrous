pub mod chunk_system;
pub mod events;
pub mod gen_test;
mod level;
pub mod logger;
pub mod randomness;
pub mod region;
pub mod terrain_gen;
pub mod world_state;
use events::{ChunkLoadTask, ServerEvent};
use region::RegionManager;
use world_state::WorldState;
mod util;
use super::packets::play::SetCenterChunk;
use super::player::Player;
use crate::JAR_RESOURCES_DIR;
pub use chunk_system::{Chunk, LudiChunkLoader};
use level::ServerLevel;
use logger::LOGGER;
use serde::de::{self, Deserializer};
use serde::{ser, Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub struct ServerData {
    pub config: ServerProperties,
    // get player by entity id
    players: RwLock<HashMap<i32, Arc<Player>>>,
    pub start_time: std::time::Instant,
    pub mappings: ServerMappings,
    pub level: ServerLevel,
    pub dimension_settings: HashMap<String, DimensionType>,
    pub dispatcher: ServerDispatcher,
    pub num_of_shards: usize,
    counters: Mutex<ServerCounters>,
}

impl std::fmt::Debug for ServerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("server").finish()
    }
}

impl ServerData {
    pub fn new() -> Self {
        let config = ServerProperties::load_properties();
        let level = ServerLevel::new(&config.level_name);
        let num_of_shards = 12;
        let dispatcher = ServerDispatcher::new(num_of_shards, &config);
        WorldState::initialize_world_states(&level.world_gen_settings);
        Self {
            level,
            dimension_settings: DimensionType::get_dimension_configs(),
            config,
            players: RwLock::new(HashMap::new()),
            start_time: std::time::Instant::now(),
            mappings: ServerMappings::load_mappings(),
            dispatcher,
            num_of_shards,
            counters: Mutex::new(ServerCounters::default()),
        }
    }

    pub fn next_eid(&self) -> u64 {
        let eid = self.counters.lock().unwrap().entity_id;
        self.counters.lock().unwrap().entity_id += 1;
        eid
    }

    pub fn get_players(&self) -> &RwLock<HashMap<i32, Arc<Player>>> {
        &self.players
    }

    pub fn add_player(&self, player: Arc<Player>) {
        self.players.write().unwrap().insert(player.id, player);
    }

    pub fn remove_player(&self, player_id: i32) {
        self.players.write().unwrap().remove(&player_id);
    }

    pub async fn load_init_chunks(&self, player: Arc<Player>, view_distance: u32) {
        let center_chunk = (
            player.chunk.0.load(Ordering::Relaxed),
            player.chunk.1.load(Ordering::Relaxed),
        );
        if let Some(tx) = player.outbound.upgrade() {
            let _ = tx.send(SetCenterChunk::new(center_chunk).into()).await;
        }
        let num_of_shards = self.num_of_shards;
        let events = tokio::task::spawn_blocking(move || {
            let new_chunks = LudiChunkLoader::calc_chunk_positions(center_chunk, view_distance);
            let shard_loads = LudiChunkLoader::shard_chunks(
                &new_chunks.iter().collect::<Vec<_>>(),
                num_of_shards,
            );
            let mut events: HashMap<usize, ChunkLoadTask> = HashMap::new();
            for load in shard_loads {
                let e = events.get_mut(&load.0);
                if let Some(task) = e {
                    task.chunks_requested.extend(&load.1);
                } else {
                    let mut new_task = ChunkLoadTask::new(player.clone());
                    new_task.chunks_requested = load.1;
                    events.insert(load.0, new_task);
                }
            }
            events
        })
        .await
        .unwrap();
        for e in events {
            self.dispatcher.shards[e.0]
                .send(ServerEvent::ChunkLoad(e.1))
                .unwrap();
        }
    }

    pub async fn load_chunks(
        &self,
        new_center_chunk: (i32, i32),
        old_center_chunk: (i32, i32),
        player: Arc<Player>,
    ) {
        if new_center_chunk.0 != old_center_chunk.0 || new_center_chunk.1 != old_center_chunk.1 {
            if let Some(tx) = player.low_priority_outbound.upgrade() {
                let _ = tx.send(SetCenterChunk::new(new_center_chunk).into()).await;
            }
            let num_of_shards = self.num_of_shards;
            let view_distance = self.config.view_distance;
            let events = tokio::task::spawn_blocking(move || {
                let new_chunks =
                    LudiChunkLoader::calc_chunk_positions(new_center_chunk, view_distance);
                let old_chunks =
                    LudiChunkLoader::calc_chunk_positions(old_center_chunk, view_distance);
                let chunk_loads = new_chunks.difference(&old_chunks).collect::<Vec<_>>();
                let chunk_unloads = old_chunks.difference(&new_chunks).collect::<Vec<_>>();
                let shard_loads = LudiChunkLoader::shard_chunks(&chunk_loads, num_of_shards);
                let shard_unloads = LudiChunkLoader::shard_chunks(&chunk_unloads, num_of_shards);
                let mut events: HashMap<usize, ChunkLoadTask> = HashMap::new();
                //println!("load e {shard_loads:?}");
                //println!("unload e {shard_unloads:?}");
                for load in shard_loads {
                    let e = events.get_mut(&load.0);
                    if let Some(task) = e {
                        task.chunks_requested.extend(&load.1);
                    } else {
                        let mut new_task = ChunkLoadTask::new(player.clone());
                        new_task.chunks_requested = load.1;
                        events.insert(load.0, new_task);
                    }
                }
                for unload in shard_unloads {
                    let e = events.get_mut(&unload.0);
                    if let Some(task) = e {
                        task.chunks_unloaded.extend(&unload.1);
                    } else {
                        let mut new_task = ChunkLoadTask::new(player.clone());
                        new_task.chunks_unloaded = unload.1;
                        events.insert(unload.0, new_task);
                    }
                }
                events
            })
            .await
            .unwrap();
            for e in events {
                self.dispatcher.shards[e.0]
                    .send(ServerEvent::ChunkLoad(e.1))
                    .unwrap();
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ServerCounters {
    pub entity_id: u64,
    pub teleport_id: u64,
}

pub struct ServerDispatcher {
    shards: Vec<UnboundedSender<ServerEvent>>,
}

impl ServerDispatcher {
    pub fn new(num_of_shards: usize, config: &ServerProperties) -> Self {
        let region_manager = RegionManager::new(config.level_name.clone(), 32);
        RegionManager::spawn_stale_checker(region_manager.clone());
        let mut shards = Vec::new();
        for i in 0..num_of_shards {
            let shard = ServerShard::new(region_manager.clone());
            let (tx, mut rx) = unbounded_channel();
            shards.push(tx);
            tokio::spawn(async move {
                let events_limit = 100;
                let mut buf = Vec::with_capacity(events_limit);
                loop {
                    let num_of_events = rx.recv_many(&mut buf, events_limit).await;
                    for event in buf.iter().take(num_of_events) {
                        shard.handle_event(event).await;
                    }
                    buf.clear();
                }
            });
        }
        Self { shards }
    }
}

struct ServerShard {
    chunk_loader: LudiChunkLoader,
}

impl ServerShard {
    pub fn new(region_manager: Arc<RegionManager>) -> Self {
        Self {
            chunk_loader: LudiChunkLoader::new(region_manager),
        }
    }

    pub async fn handle_event(&self, event: &ServerEvent) {
        match event {
            ServerEvent::ChunkLoad(task) => self.chunk_loader.handle_chunk_task(task).await,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct DimensionType {
    pub height: i32,
    pub logical_height: i32,
    pub min_y: i32,
    pub coordinate_scale: f32,
}

impl DimensionType {
    fn get_dimension_configs() -> HashMap<String, DimensionType> {
        let dir = fs::read_dir(format!("{JAR_RESOURCES_DIR}/dimension_type"))
            .unwrap_or_else(|e| panic!("Could not find dimension type configs\n{e}"));
        let mut configs = HashMap::new();
        for file in dir {
            if let Ok(file) = file {
                if file.file_type().unwrap().is_file() {
                    let mut config_file = File::open(file.path()).expect("Error opening file");
                    let mut data = String::new();
                    config_file.read_to_string(&mut data).unwrap();
                    let config =
                        serde_json::from_str(&data).expect("Error parsing dimension type config");
                    configs.insert(
                        file.file_name()
                            .into_string()
                            .unwrap()
                            .split_once(".")
                            .unwrap()
                            .0
                            .to_owned(),
                        config,
                    );
                }
            }
        }
        configs
    }
}

/// * Deprecated
/// TODO: Replace with static mappings like BLOCKSTATES at the top of `chunk_system.rs`
#[derive(Deserialize)]
pub struct ServerMappings {
    pub items: Vec<String>,
    #[serde(skip_deserializing)]
    pub items_to_numerical: Arc<HashMap<String, usize>>,
}

impl ServerMappings {
    pub fn load_mappings() -> Self {
        let mut mappings_file =
            File::open("assets/mapping-1.21.6.json").expect("assets/mapping-1.21.6.json file");
        let mut data = String::new();
        mappings_file.read_to_string(&mut data).unwrap();
        let mut mappings = serde_json::from_str::<ServerMappings>(&data)
            .expect("valid json for assets/mapping-1.21.6.json");
        mappings.items_to_numerical = Arc::new(
            mappings
                .items
                .iter()
                .enumerate()
                .map(|(i, name)| (name.clone(), i))
                .collect::<HashMap<_, _>>(),
        );
        mappings
    }
}

impl std::fmt::Debug for ServerMappings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerMappings {...}").finish()
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ServerProperties {
    pub level_name: String,
    #[serde(deserialize_with = "int_type")]
    pub network_compression_threshold: i32,
    #[serde(deserialize_with = "int_type")]
    pub view_distance: u32,
    #[serde(deserialize_with = "int_type")]
    pub server_port: u32,
    #[serde(deserialize_with = "int_type")]
    pub simulation_distance: u32,
}

impl ServerProperties {
    pub fn default() -> Self {
        ServerProperties {
            level_name: "world".into(),
            network_compression_threshold: 256,
            view_distance: 10,
            server_port: 25565,
            simulation_distance: 10,
        }
    }

    pub fn load_properties() -> Self {
        let prop_file = File::open("server.properties");
        if let Result::Err(_e) = prop_file {
            Self::create_default_properties(
                File::create_new("server.properties").expect("server.properties file"),
            )
        } else {
            let mut properties = String::new();
            prop_file.unwrap().read_to_string(&mut properties).unwrap();
            Self::from_kv(&properties).expect("server.properties file")
        }
    }

    pub fn from_kv(input: &str) -> Result<Self, serde_json::Error> {
        let map = input
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    None
                } else {
                    line.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
                }
            })
            .collect::<std::collections::HashMap<&str, &str>>();
        serde_json::from_value::<Self>(serde_json::json!(map))
    }

    pub fn create_default_properties(mut file: File) -> Self {
        LOGGER.get().unwrap().println_as(
            "No server.properties file found",
            logger::LogDomain::Server,
            logger::LogLevel::Warn,
        );
        LOGGER
            .get()
            .unwrap()
            .println("Generating default properties file...");
        let default_properties = Self::default();
        let time_parser = time::format_description::parse("[weekday repr:short] [month repr:short] [day] [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute] [year]").unwrap();
        let date_time = time::OffsetDateTime::now_local()
            .unwrap_or(time::OffsetDateTime::now_utc())
            .format(&time_parser)
            .unwrap();
        let mut default_properties_txt = "#Minecraft server properties (Generated by ludicrous)\n#"
            .to_owned()
            + &date_time
            + "\n";
        default_properties_txt =
            default_properties_txt + &default_properties.serialize(PropertiesSerializer).unwrap();
        file.write_all(&default_properties_txt.into_bytes())
            .expect("server.properties file");
        default_properties
    }
}

fn int_type<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<T>().map_err(de::Error::custom)
}

#[derive(Debug)]
pub enum Error {
    Message(String),
    Err,
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error::Message(msg.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Err => f.write_str("serde error"),
        }
    }
}

impl std::error::Error for Error {}

pub struct PropertiesSerializer;

pub struct PropertiesSerializeStruct {
    output: String,
}

impl ser::SerializeStruct for PropertiesSerializeStruct {
    type Ok = String;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let val = serde_plain::to_string(&value);
        if let Result::Err(_e) = val {
            return Err(Error::Err);
        }
        self.output.push_str(&format!("{key}={}\n", val.unwrap()));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.output)
    }
}

#[allow(unused_variables)]
impl ser::Serializer for PropertiesSerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = ser::Impossible<String, Self::Error>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeTupleStruct = Self::SerializeSeq;
    type SerializeTupleVariant = Self::SerializeSeq;
    type SerializeMap = Self::SerializeSeq;
    type SerializeStruct = PropertiesSerializeStruct;
    type SerializeStructVariant = Self::SerializeSeq;

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(PropertiesSerializeStruct {
            output: String::new(),
        })
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unimplemented!()
    }
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unimplemented!()
    }
    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!()
    }
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unimplemented!()
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unimplemented!()
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unimplemented!()
    }
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unimplemented!()
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unimplemented!()
    }
}
