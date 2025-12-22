use super::super::util::{lerp3_f64, smoothstep};
use super::functions::{BlendedNoise, OldBlendedNoiseFn};
use crate::server::util::get_noise_key;
use crate::server::world_state::WORLD_STATES;
use crate::{RandomGenerator, JAR_RESOURCES_DIR};
use ahash::{AHashMap, AHashSet};
use bytemuck::Zeroable;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use std::sync::{Mutex, OnceLock};

pub static TEMP_NOISE_INSTANCE_MAP: Lazy<Mutex<AHashMap<String, AHashSet<String>>>> =
    Lazy::new(|| Mutex::new(AHashMap::new()));

const F64_TWO_POW_25: f64 = (1 << 25) as f64;

/// For External Density Functions see `func_deserialize.rs`
pub static EXTERNAL_NOISE_INSTANCES: OnceLock<AHashMap<u64, VanillaNoise>> = OnceLock::new();

pub fn initialize_noise_instances() {
    EXTERNAL_NOISE_INSTANCES.get_or_init(|| {
        let mut noise_instances = AHashMap::new();
        for noise_inst in TEMP_NOISE_INSTANCE_MAP.lock().unwrap().iter() {
            let path = noise_inst.0.split_once(":").unwrap().1;
            let noise_file =
                fs::read_to_string(format!("{JAR_RESOURCES_DIR}/worldgen/noise/{path}.json"))
                    .unwrap_or_else(|_| {
                        panic!(
                            "Error: Could not find noise arguments .json for: {}",
                            noise_inst.0
                        )
                    });
            let noise_args = serde_json::from_str::<NoiseArguments>(&noise_file)
                .unwrap_or_else(|_| panic!("Error: Could not parse .json of: {}", noise_inst.0));
            for dimension in noise_inst.1.iter() {
                let dimension_state = WORLD_STATES.get().unwrap().get(dimension).unwrap();
                let mut rand = dimension_state
                    .random
                    .hash_to_rand(&format!("minecraft:{path}"));
                let noise = VanillaNoise::new(
                    &mut rand,
                    &noise_args,
                    !dimension_state.settings.legacy_random_source,
                );
                noise_instances.insert(get_noise_key(dimension, noise_inst.0), noise);
            }
        }
        noise_instances
    });
}

pub static OLD_BLENDED_NOISE: Lazy<BlendedNoise> = Lazy::new(OldBlendedNoiseFn::create_noise_gen);

#[derive(Debug)]
pub struct VanillaNoise {
    pub noise1: PerlinNoise,
    pub noise2: PerlinNoise,
    pub val_factor: f64,
    pub val_max: f64,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, Zeroable)]
pub struct VanillaNoiseState {
    pub noises: [PerlinNoiseState; 2],
    pub val_factor: f64,
    pub val_max: f64,
}

impl VanillaNoise {
    pub fn new(
        rand: &mut Box<dyn RandomGenerator>,
        args: &NoiseArguments,
        is_modern: bool,
    ) -> Self {
        let mut max = i32::MAX;
        let mut min = i32::MIN;
        for (i, &ampl) in args.amplitudes.iter().enumerate() {
            if ampl != 0.0 {
                max = std::cmp::min(max, i as i32);
                min = std::cmp::max(min, i as i32);
            }
        }
        let amplitudes = args.amplitudes.clone().into_boxed_slice();
        let noise1 = PerlinNoise::new(rand, args.first_octave, amplitudes.clone(), is_modern);
        let noise2 = PerlinNoise::new(rand, args.first_octave, amplitudes, is_modern);
        let val_factor = (1.0 / 6.0) / (0.1 * (1.0 + (1.0 / (min - max + 1) as f64)));
        let val_max = (noise1.get_max_val() + noise2.get_max_val()) * val_factor;
        VanillaNoise {
            noise1,
            noise2,
            val_factor,
            val_max,
        }
    }

    #[inline(always)]
    pub fn get_val(&self, x1: f64, y1: f64, z1: f64) -> f64 {
        let x2 = x1 * 1.0181268882175227;
        let y2 = y1 * 1.0181268882175227;
        let z2 = z1 * 1.0181268882175227;
        (self.noise1.get_val(x1, y1, z1) + self.noise2.get_val(x2, y2, z2)) * self.val_factor
    }

    pub fn get_max(&self) -> f64 {
        self.val_max
    }

    pub fn get_state(&self, initial_data_pos: u16) -> VanillaNoiseState {
        let mut noises = [PerlinNoiseState::default(); 2];
        noises[0] = self.noise1.get_state(initial_data_pos);
        noises[1] = self
            .noise2
            .get_state(initial_data_pos + noises[0].noise_count as u16);
        VanillaNoiseState {
            noises,
            val_factor: self.val_factor,
            val_max: self.val_max,
        }
    }

    pub fn get_all_levels(&self) -> Vec<ImprovedNoise> {
        let mut levels = vec![];
        levels.extend(self.noise1.noise_levels.clone());
        levels.extend(self.noise2.noise_levels.clone());
        levels
    }

    pub fn get_all_amplitudes(&self) -> Vec<f64> {
        let mut ampl = vec![];
        ampl.extend(self.noise1.amplitudes.clone());
        ampl.extend(self.noise2.amplitudes.clone());
        ampl
    }
}

// Deserialization for noise arguments in data/minecraft/worldgen/noise
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NoiseArguments {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

#[derive(Debug, Clone)]
pub struct PerlinNoise {
    noise_levels: Box<[ImprovedNoise]>,
    lowest_val_factor: f64,
    lowest_input_factor: f64,
    max_val: f64,
    first_octave: i32,
    amplitudes: Box<[f64]>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Zeroable)]
pub struct PerlinNoiseState {
    noise_count: u8,
    // Position of the noise level and amplitude data in a related array
    data_position: u16,
    lowest_val_factor: f64,
    lowest_input_factor: f64,
}

impl PerlinNoise {
    pub fn new(
        rand: &mut Box<dyn RandomGenerator>,
        first_octave: i32,
        amplitudes: Box<[f64]>,
        is_modern: bool,
    ) -> Self {
        let amplitude_count = amplitudes.len();
        let mut noise_levels: Box<[ImprovedNoise]> =
            vec![ImprovedNoise::default(); amplitude_count].into_boxed_slice();
        if is_modern {
            let rand_pos = rand.branch_positional();
            for (i, ampl) in amplitudes.iter().enumerate() {
                if *ampl != 0.0 {
                    noise_levels[i] = ImprovedNoise::new(
                        &mut rand_pos.hash_to_rand(&format!("octave_{}", first_octave + i as i32)),
                    );
                }
            }
        } else {
            let noise = ImprovedNoise::new(rand);
            if -first_octave >= 0 && -first_octave < amplitude_count as i32 {
                if amplitudes[-first_octave as usize] != 0.0 {
                    noise_levels[-first_octave as usize] = noise;
                }
            }

            for i in (0..(-first_octave)).rev() {
                if i < amplitude_count as i32 && amplitudes[i as usize] != 0.0 {
                    noise_levels[i as usize] = ImprovedNoise::new(rand);
                } else {
                    rand.skip(262);
                }
            }
        }
        let lowest_val_factor = (2.0f64).powf(amplitude_count as f64 - 1.0)
            / ((2.0f64).powf(amplitude_count as f64) - 1.0);
        let lowest_input_factor = (2.0f64).powf(first_octave as f64);
        let mut perlin_noise = PerlinNoise {
            noise_levels,
            lowest_val_factor,
            lowest_input_factor,
            max_val: 0.0,
            first_octave,
            amplitudes,
        };
        perlin_noise.max_val = perlin_noise.edge_val(2.0);
        perlin_noise
    }

    pub fn edge_val(&self, val: f64) -> f64 {
        let mut result = 0.0;
        let mut lowest_val = self.lowest_val_factor;

        for (i, noise) in self.noise_levels.iter().enumerate() {
            if !noise.disabled {
                result += self.amplitudes[i] * val * lowest_val;
            }
            lowest_val /= 2.0;
        }
        result
    }

    fn get_max_val(&self) -> f64 {
        self.max_val
    }

    #[inline(always)]
    pub fn get_val(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut value = 0.0;
        let mut input_factor = self.lowest_input_factor;
        let mut value_factor = self.lowest_val_factor;
        let mut i = 0;
        for noise in &self.noise_levels {
            if !noise.disabled {
                value += self.amplitudes[i]
                    * noise.generate(
                        Self::wrap(x * input_factor),
                        Self::wrap(y * input_factor),
                        Self::wrap(z * input_factor),
                        0.0,
                        0.0,
                    )
                    * value_factor;
            }
            input_factor *= 2.0;
            value_factor /= 2.0;
            i += 1;
        }
        value
    }

    #[inline(always)]
    pub fn get_noise_level(&self, level: i32) -> Option<ImprovedNoise> {
        let n_level = self.noise_levels[self.noise_levels.len() - level as usize - 1];
        if n_level.disabled {
            None
        } else {
            Some(n_level)
        }
    }

    #[inline(always)]
    pub fn get_state(&self, data_position: u16) -> PerlinNoiseState {
        PerlinNoiseState {
            noise_count: self.noise_levels.len() as u8,
            lowest_val_factor: self.lowest_val_factor,
            lowest_input_factor: self.lowest_input_factor,
            data_position,
        }
    }

    #[inline(always)]
    pub fn wrap(value: f64) -> f64 {
        value - (value / F64_TWO_POW_25 + 0.5).floor() * F64_TWO_POW_25
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable)]
pub struct ImprovedNoise {
    x: f64,
    y: f64,
    z: f64,
    values: [i32; 256],
    disabled: bool,
}

impl ImprovedNoise {
    pub fn new(rand: &mut Box<dyn RandomGenerator>) -> Self {
        let x = rand.next_f64() * 256.0;
        let y = rand.next_f64() * 256.0;
        let z = rand.next_f64() * 256.0;
        let mut values = [0i32; 256];
        for (i, val) in values.iter_mut().enumerate() {
            *val = i as i32;
        }
        for i in 0..256 as i32 {
            let t = i.wrapping_add(rand.next_i32_range(256 - i as u32)) as usize;
            if t > 255 {
                break;
            }
            values.swap(i as usize, t);
        }
        ImprovedNoise {
            x,
            y,
            z,
            values,
            disabled: false,
        }
    }

    pub fn default() -> Self {
        ImprovedNoise {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            values: [0i32; 256],
            disabled: true,
        }
    }

    #[inline(always)]
    pub fn generate(&self, mut x: f64, mut y: f64, mut z: f64, val1: f64, val2: f64) -> f64 {
        x += self.x;
        y += self.y;
        z += self.z;
        let x_floor = x.floor();
        let y_floor = y.floor();
        let z_floor = z.floor();
        x -= x_floor;
        y -= y_floor;
        z -= z_floor;
        let y_offset = if val1 != 0.0 {
            let val = if val2 >= 0.0 && val2 < y { val2 } else { y };
            y - (val1 * (((val / val1) + 1.0e-7).floor()))
        } else {
            y
        };
        self.sample_plus_lerp(
            x_floor as i32,
            y_floor as i32,
            z_floor as i32,
            x,
            y,
            z,
            y_offset,
        )
    }

    #[inline(always)]
    fn sample_plus_lerp(
        &self,
        x_floor: i32,
        y_floor: i32,
        z_floor: i32,
        x: f64,
        y: f64,
        z: f64,
        y_offset: f64,
    ) -> f64 {
        let val1 = self.values[x_floor as usize & 0xFF] & 0xFF;
        let val2 = self.values[(x_floor + 1) as usize & 0xFF] & 0xFF;
        let val3 = self.values[(y_floor + val1) as usize & 0xFF] & 0xFF;
        let val4 = self.values[(y_floor + val1 + 1) as usize & 0xFF] & 0xFF;
        let val5 = self.values[(y_floor + val2) as usize & 0xFF] & 0xFF;
        let val6 = self.values[(y_floor + val2 + 1) as usize & 0xFF] & 0xFF;

        let val7 = (self.values[(z_floor + val3) as usize & 0xFF] & 0xF) as usize;
        let val8 = (self.values[(z_floor + val5) as usize & 0xFF] & 0xF) as usize;
        let val9 = (self.values[(z_floor + val4) as usize & 0xFF] & 0xF) as usize;
        let val10 = (self.values[(z_floor + val6) as usize & 0xFF] & 0xF) as usize;
        let val11 = (self.values[(z_floor + val3 + 1) as usize & 0xFF] & 0xF) as usize;
        let val12 = (self.values[(z_floor + val5 + 1) as usize & 0xFF] & 0xF) as usize;
        let val13 = (self.values[(z_floor + val4 + 1) as usize & 0xFF] & 0xF) as usize;
        let val14 = (self.values[(z_floor + val6 + 1) as usize & 0xFF] & 0xF) as usize;

        //let x1 = Self::gradient_dot(val7, x, y_offset, z);
        let grad_x1 = SIMPLEX_GRADIENT[val7];
        let x1 = grad_x1[0] as f64 * x + grad_x1[1] as f64 * y_offset + grad_x1[2] as f64 * z;
        //let y1 = Self::gradient_dot(val8, x - 1.0, y_offset, z);
        let grad_y1 = SIMPLEX_GRADIENT[val8];
        let y1 =
            grad_y1[0] as f64 * (x - 1.0) + grad_y1[1] as f64 * y_offset + grad_y1[2] as f64 * z;
        //let x2 = Self::gradient_dot(val9, x, y_offset - 1.0, z);
        let grad_x2 = SIMPLEX_GRADIENT[val9];
        let x2 =
            grad_x2[0] as f64 * x + grad_x2[1] as f64 * (y_offset - 1.0) + grad_x2[2] as f64 * z;
        //let y2 = Self::gradient_dot(val10, x - 1.0, y_offset - 1.0, z);
        let grad_y2 = SIMPLEX_GRADIENT[val10];
        let y2 = grad_y2[0] as f64 * (x - 1.0)
            + grad_y2[1] as f64 * (y_offset - 1.0)
            + grad_y2[2] as f64 * z;
        //let x3 = Self::gradient_dot(val11, x, y_offset, z - 1.0);
        let grad_x3 = SIMPLEX_GRADIENT[val11];
        let x3 =
            grad_x3[0] as f64 * x + grad_x3[1] as f64 * y_offset + grad_x3[2] as f64 * (z - 1.0);
        //let y3 = Self::gradient_dot(val12, x - 1.0, y_offset, z - 1.0);
        let grad_y3 = SIMPLEX_GRADIENT[val12];
        let y3 = grad_y3[0] as f64 * (x - 1.0)
            + grad_y3[1] as f64 * y_offset
            + grad_y3[2] as f64 * (z - 1.0);
        //let x4 = Self::gradient_dot(val13, x, y_offset - 1.0, z - 1.0);
        let grad_x4 = SIMPLEX_GRADIENT[val13];
        let x4 = grad_x4[0] as f64 * x
            + grad_x4[1] as f64 * (y_offset - 1.0)
            + grad_x4[2] as f64 * (z - 1.0);
        //let y4 = Self::gradient_dot(val14, x - 1.0, y_offset - 1.0, z - 1.0);
        let grad_y4 = SIMPLEX_GRADIENT[val14];
        let y4 = grad_y4[0] as f64 * (x - 1.0)
            + grad_y4[1] as f64 * (y_offset - 1.0)
            + grad_y4[2] as f64 * (z - 1.0);
        lerp3_f64(
            smoothstep(x),
            smoothstep(y),
            smoothstep(z),
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
            x4,
            y4,
        )
    }

    #[inline(always)]
    fn get_noise_val(&self, i: i32) -> i32 {
        self.values[i as usize & 0xFF] & 0xFF
    }

    #[inline(always)]
    fn gradient_dot(grad: usize, x: f64, y: f64, z: f64) -> f64 {
        let grad = SIMPLEX_GRADIENT[grad as usize & 15];
        grad[0] as f64 * x + grad[1] as f64 * y + grad[2] as f64 * z
    }
}

pub const SIMPLEX_GRADIENT: [[i8; 3]; 16] = [
    [1, 1, 0],
    [-1, 1, 0],
    [1, -1, 0],
    [-1, -1, 0],
    [1, 0, 1],
    [-1, 0, 1],
    [1, 0, -1],
    [-1, 0, -1],
    [0, 1, 1],
    [0, -1, 1],
    [0, 1, -1],
    [0, -1, -1],
    [1, 1, 0],
    [0, -1, 1],
    [-1, 1, 0],
    [0, -1, -1],
];

const GRAD_X: [f64; 16] = [
    1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 0.0,
];
const GRAD_Y: [f64; 16] = [
    1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0,
];
const GRAD_Z: [f64; 16] = [
    0.0, 0.0, 0.0, 0.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 0.0, 1.0, 0.0, -1.0,
];
