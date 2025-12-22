use super::Player;
use std::sync::Arc;

pub enum ServerEvent {
    ChunkLoad(ChunkLoadTask),
}

pub struct ChunkLoadTask {
    pub player: Arc<Player>,
    pub chunks_requested: Vec<u64>,
    pub chunks_unloaded: Vec<u64>,
}

impl ChunkLoadTask {
    pub fn new(player: Arc<Player>) -> Self {
        Self {
            player,
            chunks_requested: Vec::new(),
            chunks_unloaded: Vec::new(),
        }
    }
}
