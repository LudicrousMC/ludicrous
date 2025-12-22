use super::player::Player;
use std::sync::Arc;
use tokio::sync::Mutex as AMutex;

#[derive(Debug)]
pub struct Entity {
    pub eid: i32,
    pub pos: [f64; 3],
    pub rotation: [f32; 2],
}

impl Entity {
    pub fn new(eid: i32, pos: [f64; 3], rotation: [f32; 2]) -> Self {
        Self { eid, pos, rotation }
    }
}
