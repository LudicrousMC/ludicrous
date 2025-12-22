use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ServerLevel {
    pub world_gen_settings: WorldGenSettings,
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub spawn_angle: f32,
}

impl ServerLevel {
    pub fn new(level_name: &str) -> Self {
        let level_file = File::open(format!("{level_name}/level.dat"))
            .expect("level.dat is missing in world file");

        let mut decoder = GzDecoder::new(level_file);
        let mut level_data = Vec::new();
        decoder.read_to_end(&mut level_data).unwrap();
        let data_container =
            fastnbt::from_bytes::<LevelDataNBTContainer>(&level_data).expect("Invalid level.dat");
        fastnbt::from_value(&data_container.data).expect("Invalid level.dat")
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LevelDataNBTContainer {
    data: fastnbt::Value,
}

#[derive(Deserialize, Debug)]
pub struct WorldGenSettings {
    pub dimensions: HashMap<String, WorldDimension>,
    pub seed: i64,
}

#[derive(Deserialize, Debug)]
pub struct WorldDimension {
    pub generator: WorldGenerator,
    #[serde(rename = "type")]
    pub world_type: String,
}

#[derive(Deserialize, Debug)]
pub struct WorldGenerator {
    pub biome_source: WorldBiomeSource,
    pub settings: String,
    #[serde(rename = "type")]
    pub noise_type: String,
}

#[derive(Deserialize, Debug)]
pub struct WorldBiomeSource {
    pub preset: Option<String>,
    #[serde(rename = "type")]
    pub noise_type: String,
}
