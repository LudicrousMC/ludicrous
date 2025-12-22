use super::level::{WorldDimension, WorldGenSettings};
use super::logger::LOGGER;
use super::randomness::{Xoroshiro, LCG48};
use super::terrain_gen::func_deserialize::DensityArg;
use crate::{RandomPositionalGenerator, MC_VERSION};
use ahash::AHashMap;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::io::Read;

pub static WORLD_STATES: OnceCell<AHashMap<String, WorldState>> = OnceCell::new();

#[derive(Debug)]
pub struct WorldState {
    pub random: Box<dyn RandomPositionalGenerator>,
    pub settings: NoiseSettings,
}

impl WorldState {
    pub fn new(dimension: &WorldDimension, seed: i64) -> Self {
        let file_name = dimension
            .generator
            .settings
            .split_once(":")
            .unwrap()
            .1
            .to_owned()
            + ".json";
        let mut noise_settings_file = std::fs::File::open(format!(
            "versions/{MC_VERSION}/minecraft/worldgen/noise_settings/{file_name}"
        ))
        .unwrap_or_else(|_| panic!("Could not find dimension noise settings file: {file_name}"));
        let mut data = String::new();
        noise_settings_file.read_to_string(&mut data).unwrap();
        let settings: NoiseSettings = serde_json::from_str(&data).unwrap();
        let random = if settings.legacy_random_source {
            LCG48::new(seed).branch_positional()
        } else {
            Xoroshiro::new_from_i64(seed).branch_positional()
        };
        WorldState { random, settings }
    }

    pub fn initialize_world_states(world_gen_settings: &WorldGenSettings) {
        LOGGER
            .get()
            .unwrap()
            .println("Generating World State from Seed...");
        let mut world_states = AHashMap::new();
        for (dim_name, dim_settings) in &world_gen_settings.dimensions {
            world_states.insert(
                dim_name.to_owned(),
                WorldState::new(dim_settings, world_gen_settings.seed),
            );
        }
        // Set global World States
        WORLD_STATES.set(world_states).unwrap();
        // Precompute Noise Instances
        for dimension in WORLD_STATES.get().unwrap() {
            for dense_func in dimension.1.settings.noise_router.values() {
                dense_func.precompute_noise_instance(&dimension.0);
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct NoiseSettings {
    pub aquifers_enabled: bool,
    //default_block
    //default_fluid
    pub disable_mob_generation: bool,
    pub legacy_random_source: bool,
    pub noise: NoiseBounds,
    pub noise_router: AHashMap<String, DensityArg>,
    pub ore_veins_enabled: bool,
    pub sea_level: i32,
}

#[derive(Deserialize, Debug)]
pub struct NoiseBounds {
    pub height: i32,
    pub min_y: i32,
    pub size_horizontal: u32,
    pub size_vertical: u32,
}
