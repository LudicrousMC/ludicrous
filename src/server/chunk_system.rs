use super::super::packets::play::{ChunkLightData, UnloadChunks};
use super::util::lerp_f64;
use super::{
    events::ChunkLoadTask,
    region::RegionManager,
    terrain_gen::func_deserialize::{DensityArg, DensityFnArgs},
    world_state::WORLD_STATES,
    DimensionType, Player,
};
use crate::{MAIN_DENSITY_FUNCTION, MAX_BLOCKSTATES, MC_VERSION};
use ahash::AHashMap;
use once_cell::sync::Lazy;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::sync::{atomic::AtomicU32, Arc, RwLock};
use tokio::sync::{Notify, Semaphore};

pub static BLOCKSTATES: Lazy<Vec<String>> = Lazy::new(|| {
    let mut mappings_file = File::open(&format!("assets/block-mapping-{MC_VERSION}.json"))
        .expect(&format!("assets/block-mapping-{MC_VERSION}.json file"));
    let mut data = String::new();
    mappings_file.read_to_string(&mut data).unwrap();
    serde_json::from_str::<Vec<String>>(&data).expect(&format!(
        "valid json for assets/block-mapping-{MC_VERSION}.json"
    ))
});
pub static BLOCKSTATE_MAPPINGS: Lazy<AHashMap<&'static str, u32>> = Lazy::new(|| {
    let map: AHashMap<String, u32> = BLOCKSTATES
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), i as u32))
        .collect();
    let mut fast_map = AHashMap::with_capacity(map.len());
    for (name, id) in map {
        fast_map.insert(Box::leak(name.into_boxed_str()) as &'static str, id);
    }
    fast_map
});
pub static BIOMES: Lazy<Vec<String>> = Lazy::new(|| {
    let mut mappings_file = File::open(&format!("assets/biome-mapping-{MC_VERSION}.json"))
        .expect(&format!("assets/biome-mapping-{MC_VERSION}.json file"));
    let mut data = String::new();
    mappings_file.read_to_string(&mut data).unwrap();
    serde_json::from_str::<Vec<String>>(&data).expect(&format!(
        "valid json for assets/biome-mapping-{MC_VERSION}.json"
    ))
});
pub static BIOME_MAPPINGS: Lazy<AHashMap<String, u16>> = Lazy::new(|| {
    BIOMES
        .iter()
        .enumerate()
        .map(|(i, name)| (name.clone(), i as u16))
        .collect()
});

#[derive(Debug)]
pub struct LudiChunkLoader {
    region_manager: Arc<RegionManager>,
    sampling_settings: ChunkSampleSettings,
    /// A map of player loaded chunks.
    /// Packed chunk coords (x, z) -> Chunk Data.
    loaded_chunks: HashMap<u64, Arc<ChunkLoad>>,
    load_tasks: Arc<Semaphore>,
    gen_tasks: Arc<Semaphore>,
}

impl LudiChunkLoader {
    pub fn new(region_manager: Arc<RegionManager>) -> Self {
        Self {
            region_manager,
            // Valid sampling sizes are 2, 3, 5, 9, and 17
            sampling_settings: ChunkSampleSettings::new(5, 3, 5),
            loaded_chunks: HashMap::new(),
            load_tasks: Arc::new(Semaphore::new(2)),
            gen_tasks: Arc::new(Semaphore::new(2)),
        }
    }

    pub fn pos_to_chunk(x: f64, z: f64) -> (i32, i32) {
        (
            (if x.is_sign_positive() { x } else { x - 16.0 } / 16.0) as i32,
            (if z.is_sign_positive() { z } else { z - 16.0 } / 16.0) as i32,
        )
    }

    /// returns (min_x_bound, max_x_bound, min_z_bound, max_z_bound)
    pub fn chunk_to_pos_bounds(x: i32, z: i32) -> (i32, i32, i32, i32) {
        (x * 16, x * 16 + 16, z * 16, z * 16 + 16)
    }

    pub fn chunk_to_region(x: i32, z: i32) -> (i32, i32) {
        (
            (if x.is_positive() { x } else { x - 31 } / 32),
            (if z.is_positive() { z } else { z - 31 } / 32),
        )
    }

    pub fn chunk_to_region_relative(x: i32, z: i32) -> (i32, i32) {
        (x & 31, z & 31)
    }

    pub fn chunk_idx_to_coord(idx: usize, region_coord: u64) -> (i32, i32) {
        let rel_x = idx & 31; // mod 32
        let rel_z = (idx - rel_x) >> 5; // divide by 32
        let (rx, rz) = Self::unpack_coords(region_coord);
        (rel_x as i32 + rx * 32, rel_z as i32 + rz * 32)
    }

    pub fn pack_coords(coords: (i32, i32)) -> u64 {
        ((coords.0 as u64) << 32) | coords.1 as u32 as u64
    }

    pub fn unpack_coords(packed_coords: u64) -> (i32, i32) {
        ((packed_coords >> 32) as i32, packed_coords as u32 as i32)
    }

    /// x = 26 bits, y = 12 bits, z = 26 bits
    pub fn pack_xyz(coords: (i32, i32, i32)) -> u64 {
        let x = (coords.0 + (1 << 25)) as u64 & 0x3FFFFFF;
        let y = (coords.1 + (1 << 11)) as u64 & 0xFFF;
        let z = (coords.2 + (1 << 25)) as u64 & 0x3FFFFFF;
        (x << 38) | (y << 26) | z
    }

    pub fn unpack_xyz(packed: u64) -> (i32, i32, i32) {
        let x = ((packed >> 38) & 0x3FFFFFF) as i32 - (1 << 25);
        let y = ((packed >> 26) & 0xFFF) as i32 - (1 << 11);
        let z = (packed & 0x3FFFFFF) as i32 - (1 << 25);
        (x, y, z)
    }

    pub async fn handle_chunk_task(&self, task: &ChunkLoadTask) {
        let load_semaphore = self.load_tasks.clone();
        let gen_semaphore = self.gen_tasks.clone();
        let player = task.player.clone();
        let chunkloads = task.chunks_requested.clone();
        let chunkunloads = task.chunks_unloaded.clone();
        let region_manager = self.region_manager.clone();
        let sample_settings = self.sampling_settings.clone();
        tokio::task::spawn(async move {
            let _permit = load_semaphore.acquire().await.unwrap();
            let region_map = tokio::task::spawn_blocking(move || {
                let mut region_map = HashMap::new();
                for chunk in chunkloads.iter() {
                    let (cx, cz) = Self::unpack_coords(*chunk);
                    let (rx, rz) = Self::chunk_to_region(cx, cz);
                    let entry = region_map
                        .entry(Self::pack_coords((rx, rz)))
                        .or_insert_with(Vec::new);
                    let (c_relx, c_relz) = Self::chunk_to_region_relative(cx, cz);
                    let c_idx = c_relx as usize + c_relz as usize * 32;
                    entry.push(c_idx);
                }
                region_map
            })
            .await
            .unwrap();
            if let Some(tx) = player.low_priority_outbound.upgrade() {
                let _ = tx
                    .send(UnloadChunks::new(chunkunloads.clone()).into())
                    .await;
            }
            let mut chunks_found = Vec::new();
            let mut chunks_not_found = Vec::new();
            for (region, chunk_indexes) in region_map {
                let (chunks, not_found) = region_manager
                    .get_region_chunks(1, region, chunk_indexes)
                    .await;
                chunks_found.extend(chunks);
                for c in not_found {
                    chunks_not_found.push(Self::pack_coords(Self::chunk_idx_to_coord(c, region)));
                }
                //println!("{chunks:?}");
            }
            if let Some(tx) = player.low_priority_outbound.upgrade() {
                let player = player.clone();
                let chunk_pkt =
                    tokio::task::spawn_blocking(move || ChunkLightData::new(chunks_found, player))
                        .await
                        .unwrap();
                let _ = tx.send(chunk_pkt.into()).await;
            }

            if !chunks_not_found.is_empty() {
                tokio::task::spawn(async move {
                    let permit = gen_semaphore.acquire().await.unwrap();
                    let main_dense_fn = WORLD_STATES
                        .get()
                        .unwrap_or_else(|| panic!("World state not initialized"))
                        .get("minecraft:overworld")
                        .unwrap()
                        .settings
                        .noise_router
                        .get(MAIN_DENSITY_FUNCTION)
                        .unwrap();
                    let dim_type = player
                        .server
                        .dimension_settings
                        .get("overworld")
                        .unwrap()
                        .clone();
                    let chunks_gen = tokio::task::spawn_blocking(move || {
                        let mut sample_cache = AHashMap::new();
                        let mut chunks_gen = Vec::new();
                        for chunk in chunks_not_found {
                            chunks_gen.push(Arc::new(Chunk::generate(
                                chunk,
                                "minecraft:overworld",
                                &dim_type,
                                main_dense_fn,
                                &mut sample_cache,
                                &sample_settings,
                            )));
                        }
                        chunks_gen
                    })
                    .await
                    .unwrap();
                    if let Some(tx) = player.low_priority_outbound.upgrade() {
                        let chunk_pkt = tokio::task::spawn_blocking(move || {
                            ChunkLightData::new(chunks_gen, player)
                        })
                        .await
                        .unwrap();
                        let _ = tx.send(chunk_pkt.into()).await;
                    }
                });
            }
        });
    }

    pub fn shard_chunks(chunks: &[&u64], num_of_shards: usize) -> HashMap<usize, Vec<u64>> {
        let mut map: HashMap<usize, Vec<u64>> = HashMap::new();
        for c in chunks {
            let chunk_shard = Self::hash_chunk_coord(**c, num_of_shards);
            let shard_chunks = map.get_mut(&chunk_shard);
            if let Some(chunks) = shard_chunks {
                chunks.push(**c);
            } else {
                map.insert(chunk_shard, vec![**c]);
            }
        }
        map
    }

    fn hash_chunk_coord(coord: u64, num_of_shards: usize) -> usize {
        let (x, z) = Self::unpack_coords(coord);
        let (x, z) = (x as u32, z as u32);
        let mut hasher = ahash::AHasher::default();
        x.hash(&mut hasher);
        z.hash(&mut hasher);
        (hasher.finish() as usize) % num_of_shards
    }

    pub fn calc_chunk_positions(center_chunk: (i32, i32), view_dist: u32) -> HashSet<u64> {
        let view_dist = view_dist as i32;
        let mut chunks = HashSet::new();
        for x in (-view_dist + center_chunk.0)..=(view_dist + center_chunk.0) {
            for z in (-view_dist + center_chunk.1)..=(view_dist + center_chunk.1) {
                chunks.insert(Self::pack_coords((x, z)));
            }
        }
        chunks
    }
}

/// To be used if multiple players request the same chunk while chunk is loading and to keep track
/// of chunk interest
#[derive(Default)]
pub struct ChunkLoad {
    pub load_notify: Notify,
    pub view_count: AtomicU32,
    pub players: Option<RwLock<HashMap<i32, Arc<Player>>>>,
    pub data: Option<Chunk>,
}

impl ChunkLoad {
    pub fn new(load_notify: Notify) -> Arc<Self> {
        Arc::new(Self {
            load_notify,
            view_count: AtomicU32::new(1),
            ..Default::default()
        })
    }
}

impl std::fmt::Debug for ChunkLoad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkLoadTask {...}").finish()
    }
}

#[derive(Deserialize, Default)]
pub struct Chunk {
    #[serde(rename = "Heightmaps")]
    pub heightmaps: ChunkHeightmaps,
    pub sections: Vec<ChunkSection>,
    #[serde(rename = "xPos")]
    pub x: i32,
    #[serde(rename = "yPos")]
    pub y: i8,
    #[serde(rename = "zPos")]
    pub z: i32,
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(&format!("Chunk ({}, {}, {})", self.x, self.y, self.z))
            .finish()
    }
}

impl Chunk {
    pub fn generate(
        packed_coord: u64,
        dimension: &str,
        dimension_type: &DimensionType,
        density_function: &DensityArg,
        sample_cache: &mut AHashMap<u64, f64>,
        sampling_settings: &ChunkSampleSettings,
    ) -> Self {
        let min_y = dimension_type.min_y;
        let min_section_y = (min_y as f32 / 16.0).floor() as i8;
        let max_y = dimension_type.logical_height + min_y;
        let max_section_y = (max_y as f32 / 16.0).ceil() as i8;
        let chunk_sections = (max_section_y + min_section_y.abs()) as usize;
        let chunk_coord = LudiChunkLoader::unpack_coords(packed_coord);
        // Generate chunk sections
        let mut positions_to_gen = Vec::new();
        let mut cached_positions = Vec::new();
        let mut pos_count = 0 as usize;
        {
            for y in (min_y..max_y).step_by(sampling_settings.y_sample_spacing as usize) {
                for z in ((chunk_coord.1 * 16)..=(chunk_coord.1 * 16 + 16))
                    .step_by(sampling_settings.z_sample_spacing as usize)
                {
                    for x in ((chunk_coord.0 * 16)..=(chunk_coord.0 * 16 + 16))
                        .step_by(sampling_settings.x_sample_spacing as usize)
                    {
                        let packed_coord = LudiChunkLoader::pack_xyz((x, y, z));
                        if let Some(sample) = sample_cache.get(&packed_coord) {
                            cached_positions.push((pos_count, *sample));
                        } else {
                            positions_to_gen.push(packed_coord);
                        }
                        pos_count += 1;
                    }
                }
            }
        }
        let mut args = DensityFnArgs::new_from_positions(dimension, &positions_to_gen);
        args.column_cache_passthrough = true;
        let block_states = Self::generate_chunk_blockstates(
            density_function,
            &mut args,
            chunk_sections,
            cached_positions,
            sample_cache,
            sampling_settings,
        );
        let mut sections = vec![ChunkSection::default(); chunk_sections];
        for (i, section_block_states) in block_states.into_iter().enumerate() {
            sections[i] = ChunkSection {
                y: i as i8 + min_section_y,
                biomes: ChunkBiomes::default(),
                block_states: section_block_states,
                sky_light: None,
                block_light: None,
            };
        }
        Chunk {
            /*heightmaps: ChunkHeightmaps {
                world_surface: Some(Self::encode_heightmap(world_surface)),
                ..Default::default()
            },*/
            sections,
            x: chunk_coord.0,
            y: min_section_y,
            z: chunk_coord.1,
            ..Default::default()
        }
    }

    pub fn from_data(data: &[u8]) -> Result<Self, fastnbt::error::Error> {
        fastnbt::from_bytes::<Chunk>(data)
    }

    pub fn get_heightmap_count(&self) -> i32 {
        let mut count = 0;
        let heightmaps = &self.heightmaps;
        if heightmaps.motion_blocking.is_some() {
            count += 1;
        }
        if heightmaps.motion_blocking_no_leaves.is_some() {
            count += 1;
        }
        if heightmaps.ocean_floor.is_some() {
            count += 1;
        }
        if heightmaps.world_surface.is_some() {
            count += 1;
        }
        count
    }

    fn encode_heightmap(heightmap: [[i32; 16]; 16]) -> fastnbt::LongArray {
        const BPE: usize = 9;
        let mut packed_heightmap = vec![0i64; 37];
        let mut bit_index = 0;
        for z in 0..16 {
            for x in 0..16 {
                let height = heightmap[x][z];
                let value = (height + 64).clamp(0, 511) as i64;

                let start_long = bit_index / 64;
                let start_offset = bit_index % 64;

                if start_offset + BPE <= 64 {
                    packed_heightmap[start_long] |= value << start_offset;
                } else {
                    let low_bits = 64 - start_offset;
                    packed_heightmap[start_long] |= (value & ((1 << low_bits) - 1)) << start_offset;
                    packed_heightmap[start_long + 1] |= value >> low_bits;
                }
                bit_index += BPE;
            }
        }
        fastnbt::LongArray::new(packed_heightmap)
    }

    fn find_block_height(
        block_x: i32,
        block_z: i32,
        function: &DensityArg,
        dimension: &str,
        settings: Arc<DimensionType>,
    ) -> i32 {
        let min_y = settings.min_y;
        let max_y = settings.logical_height - settings.min_y.abs();
        for block_y in (min_y..max_y).rev() {
            if function.compute(&mut DensityFnArgs::new(
                block_x, block_y, block_z, dimension,
            )) > 0.0
            {
                return block_y;
            }
        }
        min_y
    }

    #[inline]
    fn generate_chunk_blockstates(
        function: &DensityArg,
        args: &mut DensityFnArgs,
        chunk_sections: usize,
        cached_positions: Vec<(usize, f64)>,
        sample_cache: &mut AHashMap<u64, f64>,
        sampling_settings: &ChunkSampleSettings,
    ) -> Vec<ChunkBlockStates> {
        let mut densities = vec![0f64; args.slice_positions.len()];
        // Generate new position densities
        function.compute_slice(args, &mut densities);
        // Save chunk border densities to cache
        for (i, density) in densities.iter().enumerate() {
            let pos = args.slice_positions[i as usize];
            let coord = LudiChunkLoader::unpack_xyz(pos);
            let chunk_rel_x = coord.0 % 16;
            let chunk_rel_z = coord.2 % 16;
            if chunk_rel_x == 0 || chunk_rel_z == 0 {
                sample_cache.insert(pos, *density);
            }
        }
        // Update densities with cached densities
        for (index, density) in cached_positions {
            densities.insert(index, density);
        }
        let air_id = *BLOCKSTATE_MAPPINGS.get("air").unwrap();
        let stone_id = *BLOCKSTATE_MAPPINGS.get("stone").unwrap();
        let mut section_block_states = vec![ChunkBlockStates::default(); chunk_sections];
        // Chunk size is amount of blocks per chunk section (4096)
        for (section, section_state) in section_block_states.iter_mut().enumerate() {
            let mut palette: Vec<ChunkBlock> = vec![];
            let mut palette_map = [u16::MAX; MAX_BLOCKSTATES];
            let mut block_indices = vec![0u16; 4096];
            for (i, block) in block_indices.iter_mut().enumerate() {
                let i = i as i32;
                let x = i % 16;
                let y = (i / 256) + (section as i32 * 16);
                let z = (i / 16) % 16;

                let density = Self::trilinear_interpolate(&densities, x, y, z, sampling_settings);
                let block_type = if density > 0.0 { stone_id } else { air_id };
                let block_indice = palette_map.get_mut(block_type as usize).unwrap();
                *block = if *block_indice == u16::MAX {
                    let id = palette.len() as u16;
                    *block_indice = id;
                    palette.push(ChunkBlock { id: block_type });
                    id
                } else {
                    *block_indice
                };
            }
            let bpe = ((palette.len().next_power_of_two() as f32).log2().ceil() as usize).max(4);
            let data = if palette.len() == 1 {
                None
            } else {
                Some(Self::pack_block_indices(&block_indices, bpe))
            };
            *section_state = ChunkBlockStates { palette, data };
        }
        section_block_states
    }

    #[inline]
    fn pack_block_indices(data: &[u16], bits: usize) -> fastnbt::LongArray {
        let total_bits = data.len() * bits;
        let longs_needed = (total_bits + 63) / 64;
        let mut packed_blocks = vec![0i64; longs_needed];
        let mut bit_index = 0;
        for &value in data {
            let value = value as i64;
            let long_index = bit_index / 64;
            let offset = bit_index % 64;
            if offset + bits <= 64 {
                packed_blocks[long_index] |= value << offset;
            } else {
                let low_bits = 64 - offset;
                packed_blocks[long_index] |= (value & ((1 << low_bits) - 1)) << offset;
                packed_blocks[long_index + 1] |= value >> low_bits;
            }
            bit_index += bits;
        }
        fastnbt::LongArray::new(packed_blocks)
    }

    #[inline(always)]
    fn trilinear_interpolate(
        samples: &[f64],
        x: i32,
        y: i32,
        z: i32,
        sampling_settings: &ChunkSampleSettings,
    ) -> f64 {
        let x_samples = sampling_settings.num_x_samples as i32;
        let z_samples = sampling_settings.num_z_samples as i32;
        let x_sample_spacing = sampling_settings.x_sample_spacing as i32;
        let y_sample_spacing = sampling_settings.y_sample_spacing as i32;
        let z_sample_spcaing = sampling_settings.z_sample_spacing as i32;
        let sample_x = x / x_sample_spacing;
        let sample_y = y / y_sample_spacing;
        let sample_z = z / z_sample_spcaing;
        if sample_x == 0 && sample_y == 0 && sample_z == 0 {
            let index = sample_y * (z_samples * x_samples) + sample_z * x_samples + sample_x;
            return samples[index as usize];
        }
        let local_x = x % x_sample_spacing;
        let local_y = y % y_sample_spacing;
        let local_z = z % z_sample_spcaing;

        let xz_samples = z_samples * x_samples;
        let samp_x1_index = sample_x + 1;
        let samp_y_index = sample_y * xz_samples;
        let samp_y1_index = (sample_y + 1).clamp(0, 47) * xz_samples;
        let samp_z_index = sample_z * x_samples;
        let samp_z1_index = (sample_z + 1) * x_samples;

        let i000 = sample_x + samp_y_index + samp_z_index;
        let i100 = samp_x1_index + samp_y_index + samp_z_index;
        let i010 = sample_x + samp_y1_index + samp_z_index;
        let i110 = samp_x1_index + samp_y1_index + samp_z_index;
        let i001 = sample_x + samp_y_index + samp_z1_index;
        let i101 = samp_x1_index + samp_y_index + samp_z1_index;
        let i011 = sample_x + samp_y1_index + samp_z1_index;
        let i111 = samp_x1_index + samp_y1_index + samp_z1_index;

        let s000 = samples[i000 as usize];
        let s100 = samples[i100 as usize];
        let s010 = samples[i010 as usize];
        let s110 = samples[i110 as usize];
        let s001 = samples[i001 as usize];
        let s101 = samples[i101 as usize];
        let s011 = samples[i011 as usize];
        let s111 = samples[i111 as usize];
        let fx = local_x as f64 / x_sample_spacing as f64;
        let fy = local_y as f64 / y_sample_spacing as f64;
        let fz = local_z as f64 / z_sample_spcaing as f64;

        let x00 = lerp_f64(fx, s000, s100);
        let x01 = lerp_f64(fx, s001, s101);
        let x10 = lerp_f64(fx, s010, s110);
        let x11 = lerp_f64(fx, s011, s111);

        let z0 = lerp_f64(fz, x00, x01);
        let z1 = lerp_f64(fz, x10, x11);
        lerp_f64(fy, z0, z1)
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSampleSettings {
    num_x_samples: u8,
    num_y_samples: u8,
    num_z_samples: u8,
    x_sample_spacing: u8,
    y_sample_spacing: u8,
    z_sample_spacing: u8,
    samples_per_section: u8,
}

impl ChunkSampleSettings {
    pub fn new(num_x_samples: u8, num_y_samples: u8, num_z_samples: u8) -> Self {
        ChunkSampleSettings {
            num_x_samples,
            num_y_samples,
            num_z_samples,
            x_sample_spacing: 16 / (num_x_samples - 1),
            y_sample_spacing: 16 / (num_y_samples - 1),
            z_sample_spacing: 16 / (num_z_samples - 1),
            samples_per_section: num_x_samples * num_y_samples * num_z_samples,
        }
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub struct ChunkHeightmaps {
    pub motion_blocking: Option<fastnbt::LongArray>,
    pub motion_blocking_no_leaves: Option<fastnbt::LongArray>,
    pub ocean_floor: Option<fastnbt::LongArray>,
    pub world_surface: Option<fastnbt::LongArray>,
}

#[derive(Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ChunkSection {
    pub y: i8,
    #[serde(rename = "biomes")]
    pub biomes: ChunkBiomes,
    #[serde(rename = "block_states")]
    pub block_states: ChunkBlockStates,
    pub sky_light: Option<fastnbt::ByteArray>,
    pub block_light: Option<fastnbt::ByteArray>,
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct ChunkBiomes {
    #[serde(deserialize_with = "map_biome_to_id")]
    pub palette: Vec<u16>,
    pub data: Option<fastnbt::LongArray>,
}

impl ChunkBiomes {
    fn default() -> Self {
        ChunkBiomes {
            palette: vec![1; 1],
            data: None,
        }
    }
}

fn map_biome_to_id<'de, D>(deserializer: D) -> Result<Vec<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    let biomes: Vec<Cow<&str>> = Deserialize::deserialize(deserializer)?;
    biomes
        .iter()
        .map(|b| {
            BIOME_MAPPINGS
                .get(b.split_once(':').unwrap().1)
                .cloned()
                .ok_or_else(|| serde::de::Error::custom(format!("Biome type not found: {b}")))
        })
        .collect()
}

#[derive(Deserialize, Default, Debug, Clone)]
pub struct ChunkBlockStates {
    pub palette: Vec<ChunkBlock>,
    pub data: Option<fastnbt::LongArray>,
}

#[derive(Default, Debug, Clone)]
pub struct ChunkBlock {
    pub id: u32,
}

impl<'de> Deserialize<'de> for ChunkBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyDataVisitor;

        impl<'de> Visitor<'de> for MyDataVisitor {
            type Value = ChunkBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct ChunkBlock")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut name: Option<String> = None;
                let mut properties: Option<HashMap<Cow<&str>, Cow<&str>>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "Name" => name = Some(map.next_value()?),
                        "Properties" => properties = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let name = name.ok_or_else(|| serde::de::Error::missing_field("name"))?;

                let block_id = BLOCKSTATE_MAPPINGS
                    .get(deserialize_format_blockstate(&name, properties).as_str())
                    .unwrap_or_else(|| {
                        eprintln!("Error: Block not found ({name})");
                        &0
                    });
                Ok(ChunkBlock { id: *block_id })
            }
        }
        deserializer.deserialize_map(MyDataVisitor)
    }
}

pub fn deserialize_format_blockstate(
    name: &str,
    properties: Option<HashMap<Cow<&str>, Cow<&str>>>,
) -> String {
    let mut blockstate = name.split_once(':').unwrap().1.to_string();
    if let Some(properties) = properties {
        let mut blockstate_properties = properties
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>();
        blockstate_properties.sort_by_key(|k| k.clone());
        blockstate = format!("{blockstate}[{}]", blockstate_properties.join(",").as_str());
    }
    blockstate
}
