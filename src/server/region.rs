use super::chunk_system::{Chunk, LudiChunkLoader};
use dashmap::DashMap;
use flate2::read::{GzDecoder, ZlibDecoder};
use lz4::Decoder as Lz4Decoder;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug)]
pub struct RegionManager {
    pub level_name: String,
    pub start_time: std::time::Instant,
    /// Must be higher than the number of server shards to avoid a deadlock situation
    cache_capacity: usize,
    pub cache: DashMap<RegionKey, Arc<CachedRegion>>,
}

impl RegionManager {
    pub fn new(level_name: String, cache_capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            level_name,
            start_time: std::time::Instant::now(),
            // Must be higher than the number of server shards to avoid a deadlock situation
            cache_capacity,
            cache: DashMap::with_capacity(cache_capacity),
        })
    }

    pub async fn get_region_chunks(
        &self,
        dim: i32,
        coord: u64,
        chunks_relative_idx: Vec<usize>,
    ) -> (Vec<Arc<Chunk>>, Vec<usize>) {
        let key = RegionKey::new(dim, coord);
        let cached_region = self.cache.get(&key);
        let region = if let Some(r) = cached_region {
            r.in_use.fetch_add(1, Ordering::SeqCst);
            r.last_use.store(self.calc_curr_time(), Ordering::Relaxed);
            r.clone()
        } else {
            // if over capacity, evict least recently used
            // goes over capacity if no evictable lru
            if self.cache.len() >= self.cache_capacity {
                self.remove_lru();
            }
            let (rx, rz) = LudiChunkLoader::unpack_coords(coord);
            let region_file = File::open(format!("{}/region/r.{rx}.{rz}.mca", self.level_name));
            if let Ok(file) = region_file {
                let r = CachedRegion::new(file, self.calc_curr_time());
                self.cache.insert(key, r.clone());
                r
            } else {
                return (vec![], chunks_relative_idx);
            }
        };

        let region_data = region.clone();
        let (chunks, not_found) = tokio::task::spawn_blocking(move || {
            let mut not_found = Vec::new();
            let mut chunks = Vec::new();
            for chunk_idx in chunks_relative_idx {
                let location_idx = chunk_idx * 4;
                let chunk_location = &region_data.data.get(location_idx..location_idx + 4);
                if chunk_location.is_none() {
                    not_found.push(chunk_idx);
                    continue;
                }
                let chunk_location = chunk_location.unwrap();
                let offset = ((chunk_location[0] as usize) << 16)
                    | ((chunk_location[1] as usize) << 8)
                    | chunk_location[2] as usize;
                if offset == 0 || chunk_location[3] == 0 {
                    not_found.push(chunk_idx);
                    continue;
                }
                let full_chunk_data = &region_data.data.get(offset * 4096..);
                if full_chunk_data.is_none() {
                    not_found.push(chunk_idx);
                    continue;
                }
                let full_chunk_data = full_chunk_data.unwrap();
                let mut len_bytes = [0u8; 4];
                len_bytes.copy_from_slice(&full_chunk_data[0..4]);
                let length = u32::from_be_bytes(len_bytes) as usize;
                let compression_type = full_chunk_data[4];
                let raw_data = &full_chunk_data[5..(length + 5)];
                let mut data = vec![];
                match compression_type {
                    1 => {
                        GzDecoder::new(raw_data).read_to_end(&mut data).unwrap();
                    }
                    2 => {
                        ZlibDecoder::new(raw_data).read_to_end(&mut data).unwrap();
                    }
                    3 => {
                        data = raw_data.to_vec();
                    }
                    4 => {
                        Lz4Decoder::new(raw_data)
                            .unwrap()
                            .read_to_end(&mut data)
                            .unwrap();
                    }
                    _ => {
                        not_found.push(chunk_idx);
                        continue;
                    }
                }
                if data.is_empty() {
                    not_found.push(chunk_idx);
                    continue;
                }
                let chunk = Chunk::from_data(&data);
                if chunk.is_err() {
                    not_found.push(chunk_idx);
                    continue;
                }
                let chunk = Arc::new(chunk.unwrap());
                if chunk.heightmaps.world_surface.is_none() {
                    not_found.push(chunk_idx);
                    continue;
                }
                chunks.push(chunk);
            }
            (chunks, not_found)
        })
        .await
        .unwrap();

        region.in_use.fetch_sub(1, Ordering::SeqCst);
        // Signals tasks waiting for available region reading
        (chunks, not_found)
    }

    /// Remove least recently used cached region
    fn remove_lru(&self) -> bool {
        let mut del_regions = Vec::new();
        // Make list of cached regions that are not in use
        self.cache.iter().for_each(|entry| {
            if entry.in_use.load(Ordering::SeqCst) == 0 {
                del_regions.push((entry.key().clone(), entry.last_use.load(Ordering::Relaxed)));
            }
        });
        if del_regions.is_empty() {
            return false;
        }
        // Delete oldest cached region
        del_regions.sort_by_key(|(_, val)| *val);
        self.cache.remove(&del_regions.first().unwrap().0);
        true
    }

    pub fn spawn_stale_checker(manager: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                let curr_time = manager.calc_curr_time();
                manager.cache.retain(|_key, value| {
                    // If region is in use by more than 1 task or is less than 60 seconds old, keep it
                    value.in_use.load(Ordering::SeqCst) != 0
                        || curr_time - value.last_use.load(Ordering::Relaxed) < 60
                });
                // while exceeding capacity, remove least recently used
                // if there is no evictable lru, break
                while manager.cache.len() >= manager.cache_capacity {
                    if !manager.remove_lru() {
                        break;
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            }
        });
    }

    fn calc_curr_time(&self) -> u64 {
        std::time::Instant::now()
            .duration_since(self.start_time)
            .as_secs()
    }
}

#[derive(Debug)]
pub struct CachedRegion {
    pub data: Vec<u8>,
    pub in_use: AtomicUsize,
    pub last_use: AtomicU64,
}

impl CachedRegion {
    pub fn new(mut file: File, last_use: u64) -> Arc<Self> {
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        Arc::new(Self {
            data,
            in_use: AtomicUsize::new(1),
            last_use: AtomicU64::new(last_use),
        })
    }
}

#[derive(Eq, Debug, Clone)]
pub struct RegionKey {
    pub dim: i32,
    pub coord: u64,
}

impl RegionKey {
    pub fn new(dim: i32, coord: u64) -> Self {
        Self { dim, coord }
    }
}

impl PartialEq for RegionKey {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord && self.dim == other.dim
    }
}

impl Hash for RegionKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.dim.hash(state);
        self.coord.hash(state);
    }
}
