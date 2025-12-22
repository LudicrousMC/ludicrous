/**
* This module along with some methods for density functions will probably end up
* being deleted or almost completely rewritten since this code is kinda hectic and there
* is a much more optimal way of evaluating the density function through precomputing it's pipeline.
*/
use super::super::chunk_system::LudiChunkLoader;
use super::super::util::get_dir_files;
use super::{
    functions::*,
    noise_generator::{VanillaNoise, EXTERNAL_NOISE_INSTANCES},
};
use crate::server::terrain_gen::noise_generator::{
    ImprovedNoise, VanillaNoiseState, OLD_BLENDED_NOISE, TEMP_NOISE_INSTANCE_MAP,
};
use crate::server::util::get_noise_key;
use crate::MC_VERSION;
use ahash::AHasher;
use ahash::{AHashMap, AHashSet};
use bytemuck::Zeroable;
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer};
use std::cell::RefCell;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::rc::Rc;

/// For External Noise Instances see `noise_generator.rs`
pub static EXTERNAL_DENSITY_FUNCTIONS: Lazy<AHashMap<String, DensityFnType>> = Lazy::new(|| {
    let mut functions = AHashMap::new();
    let dir = fs::read_dir(format!(
        "versions/{MC_VERSION}/minecraft/worldgen/density_function"
    ))
    .expect("Could not find density functions directory");
    let mut files = Vec::new();
    get_dir_files(dir, &mut files, "").unwrap();
    for (path, mut file) in files {
        let mut data = String::new();
        file.read_to_string(&mut data).unwrap();
        let density_function = serde_json::from_str::<DensityFnType>(&data);
        /*.unwrap_or_else(|e| {
            panic!("Error deserializing external density function: {path}\n{e}")
        });*/
        if let Ok(func) = density_function {
            functions.insert("minecraft:".to_string() + &path, func);
        }
    }
    functions
});

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DensityArg {
    Constant(f64),
    DensityFn(Box<DensityFnType>),
    #[serde(deserialize_with = "lookup_density_fn")]
    ExternalDensityFn(Box<DensityFnType>),
}

fn lookup_density_fn<'de, D>(deserializer: D) -> Result<Box<DensityFnType>, D::Error>
where
    D: Deserializer<'de>,
{
    let path: String = Deserialize::deserialize(deserializer)?;
    let path = path.split_once(":").unwrap().1;
    let func_data = fs::read_to_string(format!(
        "versions/{MC_VERSION}/minecraft/worldgen/density_function/{path}.json"
    ))
    .unwrap_or_else(|_| panic!("Could not find density function, path: {path}"));
    Ok(Box::new(serde_json::from_str(&func_data).unwrap()))
}

impl DensityArg {
    #[inline(always)]
    pub fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            DensityArg::Constant(value) => *value,
            DensityArg::DensityFn(func) => func.compute(args),
            DensityArg::ExternalDensityFn(func) => func.compute(args), /*DensityArg::ExternalDensityFn(path) => EXTERNAL_DENSITY_FUNCTIONS
                                                                       .get(path)
                                                                       .unwrap_or_else(|| panic!("Error finding required density function: {path}"))
                                                                       .compute(args),*/
        }
    }

    #[inline(always)]
    pub fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        match self {
            DensityArg::Constant(value) => data.fill(*value),
            DensityArg::DensityFn(func) => func.compute_slice(args, data),
            DensityArg::ExternalDensityFn(func) => func.compute_slice(args, data), /*DensityArg::ExternalDensityFn(path) => EXTERNAL_DENSITY_FUNCTIONS
                                                                                   .get(path)
                                                                                   .unwrap_or_else(|| panic!("Error finding required density function: {path}"))
                                                                                   .compute_slice(args, data),*/
        }
    }

    #[inline(always)]
    pub fn compute_slice_keep_cache(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        match self {
            DensityArg::Constant(value) => data.fill(*value),
            DensityArg::DensityFn(func) => func.compute_slice_keep_cache(args, data),
            DensityArg::ExternalDensityFn(func) => func.compute_slice_keep_cache(args, data), /*DensityArg::ExternalDensityFn(path) => EXTERNAL_DENSITY_FUNCTIONS.get(path).unwrap_or_else(|| panic!("Error finding required density function: {path}")).compute_slice_keep_cache(args, data),*/
        }
    }

    #[inline(always)]
    pub fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            DensityArg::Constant(value) => *value,
            DensityArg::DensityFn(func) => func.get_min(args),
            DensityArg::ExternalDensityFn(func) => func.get_min(args), /*DensityArg::ExternalDensityFn(path) => EXTERNAL_DENSITY_FUNCTIONS
                                                                       .get(path)
                                                                       .unwrap_or_else(|| panic!("Error finding required density function: {path}"))
                                                                       .get_min(args),*/
        }
    }

    #[inline(always)]
    pub fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            DensityArg::Constant(value) => *value,
            DensityArg::DensityFn(func) => func.get_max(args),
            DensityArg::ExternalDensityFn(func) => func.get_max(args), /*DensityArg::ExternalDensityFn(path) => EXTERNAL_DENSITY_FUNCTIONS
                                                                       .get(path)
                                                                       .unwrap_or_else(|| panic!("Error finding required density function: {path}"))
                                                                       .get_max(args),*/
        }
    }

    pub fn get_tree_hash(&self, state: &mut AHasher) {
        match self {
            DensityArg::Constant(value) => {
                "const".hash(state);
                value.to_be_bytes().hash(state);
            }
            DensityArg::DensityFn(func) => func.get_tree_hash(state),
            DensityArg::ExternalDensityFn(func) => func.get_tree_hash(state), /*DensityArg::ExternalDensityFn(path) => {
                                                                              println!("{}", EXTERNAL_DENSITY_FUNCTIONS.len());
                                                                                      EXTERNAL_DENSITY_FUNCTIONS.get(path).unwrap_or_else(|| panic!("Error finding required density function: {path}")).get_tree_hash(state);
                                                                                  },*/
        }
    }

    pub fn precompute_noise_instance(&self, dimension: &str) {
        match self {
            DensityArg::Constant(_value) => {}
            DensityArg::DensityFn(func) => func.precompute_noise_instance(dimension),
            DensityArg::ExternalDensityFn(func) => func.precompute_noise_instance(dimension),
        }
    }

    pub fn get_max_branch_depth(&self) -> u16 {
        match self {
            DensityArg::Constant(_) => 0,
            DensityArg::DensityFn(func) => func.get_max_branch_depth() + 1,
            DensityArg::ExternalDensityFn(func) => func.get_max_branch_depth() + 1,
        }
    }

    pub fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        let current_frame = outline.selected_frame.clone();
        match self {
            DensityArg::Constant(value) => {
                outline.flow_arg_types.push(DensityOutlineArgType::Constant);
                outline.set_stack_arg(DensityOutlineArgType::Constant);
                outline.constant_args.push(*value);
                current_frame.borrow_mut().arg_num += 1;
            }
            DensityArg::DensityFn(func) => {
                outline.flow_arg_types.push(DensityOutlineArgType::Function);
                outline.set_stack_arg(DensityOutlineArgType::Function);
                func.generate_state(dimension, outline);
                outline.set_selected_frame(current_frame.clone());
            }
            DensityArg::ExternalDensityFn(func) => {
                outline.flow_arg_types.push(DensityOutlineArgType::Function);
                outline.set_stack_arg(DensityOutlineArgType::Function);
                func.generate_state(dimension, outline);
                outline.set_selected_frame(current_frame.clone());
                /*current_frame.borrow_mut().largest_slot =
                    outline.selected_frame.borrow().largest_slot + 1;
                outline.set_selected_frame(current_frame.clone());*/
            }
        }
    }

    // Generates the state of the function but only sets arguments if it's a constant
    pub fn generate_state_basic(&self, dimension: &str, outline: &mut DensityFnOutline) {
        let current_frame = outline.selected_frame.clone();
        match self {
            DensityArg::Constant(value) => {
                //outline.flow_arg_types.push(DensityOutlineArgType::Constant);
                outline.set_stack_arg(DensityOutlineArgType::Constant);
                outline.constant_args.push(*value);
                current_frame.borrow_mut().arg_num += 1;
            }
            DensityArg::DensityFn(func) | DensityArg::ExternalDensityFn(func) => {
                func.generate_state(dimension, outline);
                outline.set_selected_frame(current_frame.clone());
            }
        }
    }

    pub fn get_outline_type(&self) -> DensityOutlineArgType {
        match self {
            DensityArg::Constant(_) => DensityOutlineArgType::Constant,
            DensityArg::DensityFn(_) => DensityOutlineArgType::Function,
            DensityArg::ExternalDensityFn(_) => DensityOutlineArgType::Function,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NoiseArg {
    ExternalNoise(String),
}

impl NoiseArg {
    pub fn get_or_create(&self, dimension: &str) -> &VanillaNoise {
        match self {
            NoiseArg::ExternalNoise(noise_path) => EXTERNAL_NOISE_INSTANCES
                .get()
                .unwrap()
                .get(&get_noise_key(dimension, noise_path))
                .unwrap_or_else(|| panic!("Could not find noise: {noise_path}")),
        }
    }

    pub fn get_hash(&self, state: &mut AHasher) {
        match self {
            NoiseArg::ExternalNoise(noise_path) => noise_path.hash(state),
        }
    }

    /// This method is primarily called by world_state to gather all noise instances to be loaded
    /// in TEMP_NOISE_INSTANCE_MAP. After WORLD_STATES is initialized, EXTERNAL_NOISE_INSTANCES is
    /// initialized using the TEMP_NOISE_INSTANCE_MAP
    pub fn precompute_noise_instance(&self, dimension: &str) {
        match self {
            NoiseArg::ExternalNoise(noise_path) => {
                TEMP_NOISE_INSTANCE_MAP
                    .lock()
                    .unwrap()
                    .entry(noise_path.to_string())
                    .and_modify(|e| {
                        e.insert(dimension.to_string());
                    })
                    .or_insert(AHashSet::from([dimension.to_string()]));
            }
        }
    }

    pub fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        match self {
            NoiseArg::ExternalNoise(noise_path) => {
                let noise = EXTERNAL_NOISE_INSTANCES
                    .get()
                    .unwrap()
                    .get(&get_noise_key(dimension, noise_path))
                    .unwrap_or_else(|| panic!("Could not find noise: {noise_path}"));
                outline
                    .noise_states
                    .push(noise.get_state(outline.noise_levels.len() as u16));
                outline.noise_levels.extend(noise.get_all_levels());
                outline.noise_amplitudes.extend(noise.get_all_amplitudes());
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct DensityFnArgs<'a> {
    pub block_x: i32,
    pub block_y: i32,
    pub block_z: i32,
    pub dimension: &'a str,
    /// One time cache for function trees with equivalent hash fingerprints
    pub once_cache_pos: Option<u64>,
    pub once_cache: Rc<RefCell<AHashMap<u64, f64>>>,
    /// Cache for a chunk's (x, z) plane scoped to the y-axis
    pub flat_cache_level: Option<i16>,
    pub flat_cache: Rc<RefCell<AHashMap<u128, f64>>>,
    /// Will not attempt to cache if set to true
    pub flat_cache_passthrough: bool,
    /// Cache for column (x, z) for all y's in column
    pub column_cache_pos: Option<u64>,
    pub column_cache: Rc<RefCell<AHashMap<u64, f64>>>,
    pub column_cache_passthrough: bool,
    /// A Vec of packked coords (x, y, z) used for context in compute_slice methods
    pub slice_positions: &'a [u64],
    pub c_count: u64,
}

impl<'a> DensityFnArgs<'a> {
    /// Only use when evaluating slice methods
    pub fn new(block_x: i32, block_y: i32, block_z: i32, dimension: &'a str) -> Self {
        DensityFnArgs {
            block_x,
            block_y,
            block_z,
            dimension,
            ..Default::default()
        }
    }

    /// Only use when evaluating compute_slice methods
    pub fn new_from_positions(dimension: &'a str, slice_positions: &'a [u64]) -> Self {
        DensityFnArgs {
            dimension,
            slice_positions,
            ..Default::default()
        }
    }

    pub fn get_slice_args(&self, position: usize) -> Self {
        let coords = LudiChunkLoader::unpack_xyz(self.slice_positions[position]);
        DensityFnArgs {
            block_x: coords.0,
            block_y: coords.1,
            block_z: coords.2,
            once_cache: self.once_cache.clone(),
            flat_cache: self.flat_cache.clone(),
            column_cache: self.column_cache.clone(),
            ..*self
        }
    }

    pub fn mutate_coord(&mut self, block_x: i32, block_y: i32, block_z: i32) {
        self.block_x = block_x;
        self.block_y = block_y;
        self.block_z = block_z;
    }

    pub fn mutate_coord_from_slice(&mut self, slice_pos: usize) {
        let coords = LudiChunkLoader::unpack_xyz(self.slice_positions[slice_pos]);
        self.block_x = coords.0;
        self.block_y = coords.1;
        self.block_z = coords.2;
    }

    pub fn get_slice_args_new(&self, position: usize) -> Self {
        let coords = LudiChunkLoader::unpack_xyz(self.slice_positions[position]);
        DensityFnArgs {
            block_x: coords.0,
            block_y: coords.1,
            block_z: coords.2,
            once_cache: Rc::new(RefCell::new(AHashMap::new())),
            flat_cache: Rc::new(RefCell::new(AHashMap::new())),
            column_cache: Rc::new(RefCell::new(AHashMap::new())),
            ..*self
        }
    }

    pub fn get_pos_args(&self, block_x: i32, block_y: i32, block_z: i32) -> Self {
        DensityFnArgs {
            block_x,
            block_y,
            block_z,
            once_cache: self.once_cache.clone(),
            flat_cache: self.flat_cache.clone(),
            column_cache: self.column_cache.clone(),
            ..*self
        }
    }

    pub fn get_pos_args_new(&self, block_x: i32, block_y: i32, block_z: i32) -> Self {
        DensityFnArgs {
            block_x,
            block_y,
            block_z,
            once_cache: Rc::new(RefCell::new(AHashMap::new())),
            flat_cache: Rc::new(RefCell::new(AHashMap::new())),
            column_cache: Rc::new(RefCell::new(AHashMap::new())),
            ..*self
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DensityFnType {
    #[serde(rename = "minecraft:add")]
    Add(AddFn),
    #[serde(rename = "minecraft:mul")]
    Mul(MulFn),
    #[serde(rename = "minecraft:min")]
    Min(MinFn),
    #[serde(rename = "minecraft:max")]
    Max(MaxFn),
    #[serde(rename = "minecraft:abs")]
    Abs(AbsFn),
    #[serde(rename = "minecraft:square")]
    Square(SquareFn),
    #[serde(rename = "minecraft:cube")]
    Cube(CubeFn),
    #[serde(rename = "minecraft:half_negative")]
    HalfNegative(HalfNegativeFn),
    #[serde(rename = "minecraft:quarter_negative")]
    QuarterNegative(QuarterNegativeFn),
    #[serde(rename = "minecraft:squeeze")]
    Squeeze(SqueezeFn),
    #[serde(rename = "minecraft:clamp")]
    Clamp(ClampFn),
    #[serde(rename = "minecraft:y_clamped_gradient")]
    YClampedGradient(YClampedGradientFn),
    #[serde(rename = "minecraft:range_choice")]
    RangeChoice(RangeChoiceFn),
    #[serde(rename = "minecraft:noise")]
    Noise(NoiseFn),
    #[serde(rename = "minecraft:shifted_noise")]
    ShiftedNoise(ShiftedNoiseFn),
    #[serde(rename = "minecraft:spline")]
    Spline(SplineFn),
    #[serde(rename = "minecraft:weird_scaled_sampler")]
    WeirdScaledSampler(WeirdScaledSamplerFn),
    #[serde(rename = "minecraft:interpolated")]
    Interpolated(InterpolatedFn),
    #[serde(rename = "minecraft:blend_density")]
    BlendDensity(BlendDensityFn),
    #[serde(rename = "minecraft:blend_offset")]
    BlendOffset(BlendOffsetFn),
    #[serde(rename = "minecraft:blend_alpha")]
    BlendAlpha(BlendAlphaFn),
    #[serde(rename = "minecraft:cache_once")]
    CacheOnce(CacheOnceFn),
    #[serde(rename = "minecraft:flat_cache")]
    FlatCache(FlatCacheFn),
    #[serde(rename = "minecraft:cache_2d")]
    Cache2D(Cache2DFn),
    #[serde(rename = "minecraft:shift_a")]
    ShiftA(ShiftAFn),
    #[serde(rename = "minecraft:shift_b")]
    ShiftB(ShiftBFn),
    #[serde(rename = "minecraft:old_blended_noise")]
    OldBlendedNoise(OldBlendedNoiseFn),
    #[serde(rename = "minecraft:end_islands")]
    EndIslands(EndIslandsFn),
}

#[derive(Deserialize)]
pub struct CacheFnHelper {
    pub argument: DensityArg,
}

impl DensityFnType {
    #[inline(always)]
    pub fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            DensityFnType::Add(x) => x.compute(args),
            DensityFnType::Mul(x) => x.compute(args),
            DensityFnType::Min(x) => x.compute(args),
            DensityFnType::Max(x) => x.compute(args),
            DensityFnType::Abs(x) => x.compute(args),
            DensityFnType::Square(x) => x.compute(args),
            DensityFnType::Cube(x) => x.compute(args),
            DensityFnType::HalfNegative(x) => x.compute(args),
            DensityFnType::QuarterNegative(x) => x.compute(args),
            DensityFnType::Squeeze(x) => x.compute(args),
            DensityFnType::Clamp(x) => x.compute(args),
            DensityFnType::YClampedGradient(x) => x.compute(args),
            DensityFnType::RangeChoice(x) => x.compute(args),
            DensityFnType::Noise(x) => x.compute(args),
            DensityFnType::ShiftedNoise(x) => x.compute(args),
            DensityFnType::Spline(x) => x.compute(args),
            DensityFnType::WeirdScaledSampler(x) => x.compute(args),
            DensityFnType::Interpolated(x) => x.compute(args),
            DensityFnType::BlendDensity(x) => x.compute(args),
            DensityFnType::BlendOffset(x) => 0.0,
            DensityFnType::BlendAlpha(x) => 1.0,
            DensityFnType::CacheOnce(x) => x.compute(args),
            DensityFnType::FlatCache(x) => x.compute(args),
            DensityFnType::Cache2D(x) => x.compute(args),
            DensityFnType::ShiftA(x) => x.compute(args),
            DensityFnType::ShiftB(x) => x.compute(args),
            DensityFnType::OldBlendedNoise(x) => x.compute(args),
            DensityFnType::EndIslands(x) => 1.0, // unimplemented
        }
    }

    #[inline(always)]
    pub fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        match self {
            DensityFnType::Add(x) => x.compute_slice(args, data),
            DensityFnType::Mul(x) => x.compute_slice(args, data),
            DensityFnType::Min(x) => x.compute_slice(args, data),
            DensityFnType::Max(x) => x.compute_slice(args, data),
            DensityFnType::Abs(x) => x.compute_slice(args, data),
            DensityFnType::Square(x) => x.compute_slice(args, data),
            DensityFnType::Cube(x) => x.compute_slice(args, data),
            DensityFnType::HalfNegative(x) => x.compute_slice(args, data),
            DensityFnType::QuarterNegative(x) => x.compute_slice(args, data),
            DensityFnType::Squeeze(x) => x.compute_slice(args, data),
            DensityFnType::Clamp(x) => x.compute_slice(args, data),
            DensityFnType::YClampedGradient(x) => x.compute_slice(args, data),
            DensityFnType::RangeChoice(x) => x.compute_slice(args, data),
            DensityFnType::Noise(x) => x.compute_slice(args, data),
            DensityFnType::ShiftedNoise(x) => x.compute_slice(args, data),
            DensityFnType::Spline(x) => x.compute_slice(args, data),
            DensityFnType::WeirdScaledSampler(x) => x.compute_slice(args, data),
            DensityFnType::Interpolated(x) => x.compute_slice(args, data),
            DensityFnType::BlendDensity(x) => x.compute_slice(args, data),
            DensityFnType::BlendOffset(x) => data.fill(0.0),
            DensityFnType::BlendAlpha(x) => data.fill(1.0),
            DensityFnType::CacheOnce(x) => x.compute_slice(args, data),
            DensityFnType::FlatCache(x) => x.compute_slice(args, data),
            DensityFnType::Cache2D(x) => x.compute_slice(args, data),
            DensityFnType::ShiftA(x) => x.compute_slice(args, data),
            DensityFnType::ShiftB(x) => x.compute_slice(args, data),
            DensityFnType::OldBlendedNoise(x) => x.compute_slice(args, data),
            DensityFnType::EndIslands(x) => unimplemented!("end islands"),
        }
    }

    #[inline(always)]
    pub fn compute_slice_keep_cache(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        match self {
            DensityFnType::Add(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Mul(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Min(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Max(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Abs(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Square(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Cube(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::HalfNegative(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::QuarterNegative(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Squeeze(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Clamp(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::YClampedGradient(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::RangeChoice(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Noise(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::ShiftedNoise(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Spline(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::WeirdScaledSampler(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Interpolated(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::BlendDensity(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::BlendOffset(x) => data.fill(0.0),
            DensityFnType::BlendAlpha(x) => data.fill(1.0),
            DensityFnType::CacheOnce(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::FlatCache(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::Cache2D(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::ShiftA(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::ShiftB(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::OldBlendedNoise(x) => x.compute_slice_keep_cache(args, data),
            DensityFnType::EndIslands(x) => unimplemented!("end islands"),
        }
    }

    #[inline(always)]
    pub fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            DensityFnType::Add(x) => x.get_min(args),
            DensityFnType::Mul(x) => x.get_min(args),
            DensityFnType::Min(x) => x.get_min(args),
            DensityFnType::Max(x) => x.get_min(args),
            DensityFnType::Abs(x) => x.get_min(args),
            DensityFnType::Square(x) => x.get_min(args),
            DensityFnType::Cube(x) => x.get_min(args),
            DensityFnType::HalfNegative(x) => x.get_min(args),
            DensityFnType::QuarterNegative(x) => x.get_min(args),
            DensityFnType::Squeeze(x) => x.get_min(args),
            DensityFnType::Clamp(x) => x.get_min_no_args(),
            DensityFnType::YClampedGradient(x) => x.get_min_no_args(),
            DensityFnType::RangeChoice(x) => x.get_min(args),
            DensityFnType::Noise(x) => x.get_min(args),
            DensityFnType::ShiftedNoise(x) => x.get_min(args),
            DensityFnType::Spline(x) => x.get_min(args),
            DensityFnType::WeirdScaledSampler(x) => 0.0,
            DensityFnType::Interpolated(x) => x.get_min(args),
            DensityFnType::BlendDensity(x) => f64::NEG_INFINITY,
            DensityFnType::BlendOffset(x) => 0.0,
            DensityFnType::BlendAlpha(x) => 1.0,
            DensityFnType::CacheOnce(x) => x.get_min(args),
            DensityFnType::FlatCache(x) => x.get_min(args),
            DensityFnType::Cache2D(x) => x.get_min(args),
            DensityFnType::ShiftA(x) => x.get_min(args),
            DensityFnType::ShiftB(x) => x.get_min(args),
            DensityFnType::OldBlendedNoise(x) => x.get_min(args),
            DensityFnType::EndIslands(x) => unimplemented!("end islands"),
        }
    }

    #[inline(always)]
    pub fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        match self {
            DensityFnType::Add(x) => x.get_max(args),
            DensityFnType::Mul(x) => x.get_max(args),
            DensityFnType::Min(x) => x.get_max(args),
            DensityFnType::Max(x) => x.get_max(args),
            DensityFnType::Abs(x) => x.get_max(args),
            DensityFnType::Square(x) => x.get_max(args),
            DensityFnType::Cube(x) => x.get_max(args),
            DensityFnType::HalfNegative(x) => x.get_max(args),
            DensityFnType::QuarterNegative(x) => x.get_max(args),
            DensityFnType::Squeeze(x) => x.get_max(args),
            DensityFnType::Clamp(x) => x.get_max_no_args(),
            DensityFnType::YClampedGradient(x) => x.get_max_no_args(),
            DensityFnType::RangeChoice(x) => x.get_max(args),
            DensityFnType::Noise(x) => x.get_max(args),
            DensityFnType::ShiftedNoise(x) => x.get_max(args),
            DensityFnType::Spline(x) => x.get_max(args),
            DensityFnType::WeirdScaledSampler(x) => x.get_max(args),
            DensityFnType::Interpolated(x) => x.get_max(args),
            DensityFnType::BlendDensity(x) => f64::INFINITY,
            DensityFnType::BlendOffset(x) => 0.0,
            DensityFnType::BlendAlpha(x) => 1.0,
            DensityFnType::CacheOnce(x) => x.get_max(args),
            DensityFnType::FlatCache(x) => x.get_max(args),
            DensityFnType::Cache2D(x) => x.get_max(args),
            DensityFnType::ShiftA(x) => x.get_max(args),
            DensityFnType::ShiftB(x) => x.get_max(args),
            DensityFnType::OldBlendedNoise(x) => x.get_max(args),
            DensityFnType::EndIslands(x) => unimplemented!("end islands"),
        }
    }

    pub fn get_tree_hash(&self, state: &mut AHasher) {
        match self {
            DensityFnType::Add(x) => x.get_tree_hash(state),
            DensityFnType::Mul(x) => x.get_tree_hash(state),
            DensityFnType::Min(x) => x.get_tree_hash(state),
            DensityFnType::Max(x) => x.get_tree_hash(state),
            DensityFnType::Abs(x) => x.get_tree_hash(state),
            DensityFnType::Square(x) => x.get_tree_hash(state),
            DensityFnType::Cube(x) => x.get_tree_hash(state),
            DensityFnType::HalfNegative(x) => x.get_tree_hash(state),
            DensityFnType::QuarterNegative(x) => x.get_tree_hash(state),
            DensityFnType::Squeeze(x) => x.get_tree_hash(state),
            DensityFnType::Clamp(x) => x.get_tree_hash(state),
            DensityFnType::YClampedGradient(x) => x.get_tree_hash(state),
            DensityFnType::RangeChoice(x) => x.get_tree_hash(state),
            DensityFnType::Noise(x) => x.get_tree_hash(state),
            DensityFnType::ShiftedNoise(x) => x.get_tree_hash(state),
            DensityFnType::Spline(x) => x.get_tree_hash(state),
            DensityFnType::WeirdScaledSampler(x) => x.get_tree_hash(state),
            DensityFnType::Interpolated(x) => x.get_tree_hash(state),
            DensityFnType::BlendDensity(x) => x.get_tree_hash(state),
            DensityFnType::BlendOffset(x) => "blend_offset".hash(state),
            DensityFnType::BlendAlpha(x) => "blend_alpha".hash(state),
            DensityFnType::CacheOnce(x) => x.get_tree_hash(state),
            DensityFnType::FlatCache(x) => x.get_tree_hash(state),
            DensityFnType::Cache2D(x) => x.get_tree_hash(state),
            DensityFnType::ShiftA(x) => x.get_tree_hash(state),
            DensityFnType::ShiftB(x) => x.get_tree_hash(state),
            DensityFnType::OldBlendedNoise(x) => x.get_tree_hash(state),
            DensityFnType::EndIslands(x) => {} // unimplemented
        }
    }

    pub fn precompute_noise_instance(&self, dimension: &str) {
        match self {
            DensityFnType::Add(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Mul(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Min(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Max(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Abs(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Square(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Cube(x) => x.precompute_noise_instance(dimension),
            DensityFnType::HalfNegative(x) => x.precompute_noise_instance(dimension),
            DensityFnType::QuarterNegative(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Squeeze(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Clamp(x) => x.precompute_noise_instance(dimension),
            DensityFnType::YClampedGradient(x) => {}
            DensityFnType::RangeChoice(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Noise(x) => x.precompute_noise_instance(dimension),
            DensityFnType::ShiftedNoise(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Spline(x) => x.precompute_noise_instance(dimension),
            DensityFnType::WeirdScaledSampler(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Interpolated(x) => x.precompute_noise_instance(dimension),
            DensityFnType::BlendDensity(x) => x.precompute_noise_instance(dimension),
            DensityFnType::BlendOffset(x) => {}
            DensityFnType::BlendAlpha(x) => {}
            DensityFnType::CacheOnce(x) => x.precompute_noise_instance(dimension),
            DensityFnType::FlatCache(x) => x.precompute_noise_instance(dimension),
            DensityFnType::Cache2D(x) => x.precompute_noise_instance(dimension),
            DensityFnType::ShiftA(x) => x.precompute_noise_instance(dimension),
            DensityFnType::ShiftB(x) => x.precompute_noise_instance(dimension),
            DensityFnType::OldBlendedNoise(x) => {}
            DensityFnType::EndIslands(x) => {} // unimplemented
        }
    }

    pub fn get_max_branch_depth(&self) -> u16 {
        match self {
            DensityFnType::Add(x) => x.get_max_branch_depth(),
            _ => 0,
            /*DensityFnType::Mul(x) => x.get_max(args),
            DensityFnType::Min(x) => x.get_max(args),
            DensityFnType::Max(x) => x.get_max(args),
            DensityFnType::Abs(x) => x.get_max(args),
            DensityFnType::Square(x) => x.get_max(args),
            DensityFnType::Cube(x) => x.get_max(args),
            DensityFnType::HalfNegative(x) => x.get_max(args),
            DensityFnType::QuarterNegative(x) => x.get_max(args),
            DensityFnType::Squeeze(x) => x.get_max(args),
            DensityFnType::Clamp(x) => x.get_max_no_args(),
            DensityFnType::YClampedGradient(x) => x.get_max_no_args(),
            DensityFnType::RangeChoice(x) => x.get_max(args),
            DensityFnType::Noise(x) => x.get_max(args),
            DensityFnType::ShiftedNoise(x) => x.get_max(args),
            DensityFnType::Spline(x) => x.get_max(args),
            DensityFnType::WeirdScaledSampler(x) => x.get_max(args),
            DensityFnType::Interpolated(x) => x.get_max(args),
            DensityFnType::BlendDensity(x) => f64::INFINITY,
            DensityFnType::BlendOffset(x) => 0.0,
            DensityFnType::BlendAlpha(x) => 1.0,
            DensityFnType::CacheOnce(x) => x.get_max(args),
            DensityFnType::FlatCache(x) => x.get_max(args),
            DensityFnType::Cache2D(x) => x.get_max(args),
            DensityFnType::ShiftA(x) => x.get_max(args),
            DensityFnType::ShiftB(x) => x.get_max(args),
            DensityFnType::OldBlendedNoise(x) => x.get_max(args),
            DensityFnType::EndIslands(x) => unimplemented!("end islands"),*/
        }
    }

    pub fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        match self {
            DensityFnType::Add(x) => x.generate_state(dimension, outline),
            DensityFnType::Mul(x) => x.generate_state(dimension, outline),
            DensityFnType::Min(x) => x.generate_state(dimension, outline),
            DensityFnType::Max(x) => x.generate_state(dimension, outline),
            DensityFnType::Abs(x) => x.generate_state(dimension, outline),
            DensityFnType::Square(x) => x.generate_state(dimension, outline),
            DensityFnType::Cube(x) => x.generate_state(dimension, outline),
            DensityFnType::HalfNegative(x) => x.generate_state(dimension, outline),
            DensityFnType::QuarterNegative(x) => x.generate_state(dimension, outline),
            DensityFnType::Squeeze(x) => x.generate_state(dimension, outline),
            DensityFnType::Clamp(x) => x.generate_state(dimension, outline),
            DensityFnType::YClampedGradient(x) => x.generate_state(dimension, outline),
            DensityFnType::RangeChoice(x) => x.generate_state(dimension, outline),
            DensityFnType::Noise(x) => x.generate_state(dimension, outline),
            DensityFnType::ShiftedNoise(x) => x.generate_state(dimension, outline),
            DensityFnType::Spline(x) => x.generate_state(dimension, outline),
            DensityFnType::WeirdScaledSampler(x) => x.generate_state(dimension, outline),
            DensityFnType::Interpolated(x) => x.generate_state(dimension, outline),
            DensityFnType::BlendDensity(x) => x.generate_state(dimension, outline),
            DensityFnType::BlendOffset(x) => {
                outline.new_stack_frame(DensityFnOutlineType::BlendOffset)
            }
            DensityFnType::BlendAlpha(x) => {
                outline.new_stack_frame(DensityFnOutlineType::BlendAlpha)
            }
            DensityFnType::CacheOnce(x) => x.generate_state(dimension, outline),
            DensityFnType::FlatCache(x) => x.generate_state(dimension, outline),
            DensityFnType::Cache2D(x) => x.generate_state(dimension, outline),
            DensityFnType::ShiftA(x) => x.generate_state(dimension, outline),
            DensityFnType::ShiftB(x) => x.generate_state(dimension, outline),
            DensityFnType::OldBlendedNoise(x) => x.generate_state(dimension, outline),
            DensityFnType::EndIslands(x) => {
                outline.function_flow.push(DensityFnOutlineType::EndIslands)
            }
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Zeroable)]
pub struct FunctionStackFrame {
    pub fn_type: u8,
    pub reg_position: u8,
    pub arg_types: [u8; 3],
    pub arg_positions: [u16; 3],
    pub constants_index: u16,
    pub noise_index: u8,
}

#[derive(Default, Debug)]
pub struct FunctionFrameData {
    index: u16,
    pub arg_num: u8,
    pub largest_slot: u8,
    pub account_for_prev_frame: bool,
}

/// Keep track of location in function flow inside kernel function so you know what arguments to
/// access
#[derive(Default, Debug)]
pub struct DensityFnOutline {
    pub stack: Vec<FunctionStackFrame>,
    pub selected_frame: Rc<RefCell<FunctionFrameData>>,
    /// The largest slot index that will be used for computation
    pub max_slot_position: u8,
    /// Map of noise generators to prevent duplicate generators. hash => noise_states index
    pub noise_generator_map: AHashMap<u64, usize>,
    /// Int enum representing type of density function. ex "minecraft:add" = 0
    pub function_flow: Vec<DensityFnOutlineType>,
    /// Int enum representing types of arguments for density function in function_flow.
    /// ex. Read from constant_args if 0 (Constant) or read from function_flow if 1 (Function).
    pub flow_arg_types: Vec<DensityOutlineArgType>,
    /// Constant numbers used in function evaluation
    pub constant_args: Vec<f64>,
    /// Noise states for the noise density function
    pub noise_states: Vec<VanillaNoiseState>,
    /// Noise levels for vanilla noise
    pub noise_levels: Vec<ImprovedNoise>,
    /// Noise amplitudes for vanilla noise
    pub noise_amplitudes: Vec<f64>,
}

impl DensityFnOutline {
    pub fn add_blended_noise_generator(&mut self) {
        // Fit Perlin min / max limit and generator into Vanilla Noise Generator
        let vanilla_noise = VanillaNoise {
            noise1: OLD_BLENDED_NOISE.min_limit.clone(),
            noise2: OLD_BLENDED_NOISE.generator.clone(),
            val_factor: 0.0,
            val_max: 0.0,
        };
        let blended_noise_state = vanilla_noise.get_state(0);
        let blended_noise_levels = vanilla_noise.get_all_levels();
        let blended_noise_amplitudes = vanilla_noise.get_all_amplitudes();
        self.noise_states.push(blended_noise_state);
        self.noise_levels.extend(blended_noise_levels);
        self.noise_amplitudes.extend(blended_noise_amplitudes);
    }

    pub fn fix_invalid_buffers(&mut self) {
        if self.stack.is_empty() {
            self.stack.push(FunctionStackFrame::default());
        }
        if self.flow_arg_types.is_empty() {
            self.flow_arg_types.push(DensityOutlineArgType::Constant);
        }
        if self.constant_args.is_empty() {
            self.constant_args.push(0.0);
        }
        if self.noise_states.is_empty() {
            self.noise_states.push(VanillaNoiseState::default());
        }
        if self.noise_levels.is_empty() {
            self.noise_levels.push(ImprovedNoise::default());
        }
        if self.noise_amplitudes.is_empty() {
            self.noise_amplitudes.push(0.0);
        }
    }

    pub fn new_stack_frame(&mut self, fn_type: DensityFnOutlineType) {
        let reg_position = if self.stack.is_empty() {
            0
        } else {
            self.selected_frame.borrow().largest_slot
            /*self.stack
            .get(self.selected_frame.borrow().index as usize)
            .unwrap()
            .arg_positions[self.selected_frame.borrow().arg_num as usize] as u8*/
        };
        self.new_stack_frame_with_slot_pos(fn_type, reg_position);
    }

    /// Crates a new stack frame and take into account slot usage of previous branch
    pub fn new_stack_frame_with_prev_frame(&mut self, fn_type: DensityFnOutlineType) {
        // Set this frame as a arg of the previous frame
        self.set_stack_arg(DensityOutlineArgType::Function);
        let prev_max_slot = self.selected_frame.borrow().largest_slot;
        self.new_stack_frame_with_slot_pos(fn_type, prev_max_slot);
    }

    pub fn new_stack_frame_with_slot_pos(&mut self, fn_type: DensityFnOutlineType, slot_pos: u8) {
        let use_prev_largest_slot = self.selected_frame.borrow().account_for_prev_frame;
        let prev_largest_slot = self.selected_frame.borrow().largest_slot;
        // Set selected function frame
        self.selected_frame = Rc::new(RefCell::new(FunctionFrameData {
            index: self.stack.len() as u16,
            account_for_prev_frame: fn_type == DensityFnOutlineType::ShiftedNoise
                || fn_type == DensityFnOutlineType::RangeChoice
                || fn_type == DensityFnOutlineType::Spline,
            largest_slot: if use_prev_largest_slot {
                prev_largest_slot
            } else {
                0
            },
            ..Default::default()
        }));
        let constants_index = match fn_type {
            DensityFnOutlineType::Min
            | DensityFnOutlineType::Max
            | DensityFnOutlineType::Clamp
            | DensityFnOutlineType::YClampedGradient
            | DensityFnOutlineType::RangeChoice
            | DensityFnOutlineType::Noise
            | DensityFnOutlineType::ShiftedNoise
            | DensityFnOutlineType::Spline
            | DensityFnOutlineType::WeirdScaledSampler
            | DensityFnOutlineType::OldBlendedNoise => self.constant_args.len() as u16,
            _ => 0,
        };

        self.stack.push(FunctionStackFrame {
            fn_type: fn_type as u8,
            reg_position: slot_pos,
            constants_index,
            ..Default::default()
        });
    }

    pub fn set_selected_frame(&mut self, frame_data: Rc<RefCell<FunctionFrameData>>) {
        frame_data.borrow_mut().arg_num += 1;
        if frame_data.borrow().account_for_prev_frame {
            let current_largest = frame_data.borrow().largest_slot;
            frame_data.borrow_mut().largest_slot = self
                .selected_frame
                .borrow()
                .largest_slot
                .max(current_largest)
                + 1;
        } else {
            frame_data.borrow_mut().largest_slot = self.selected_frame.borrow().largest_slot + 1;
        }
        self.selected_frame = frame_data;
    }

    pub fn set_stack_arg(&mut self, arg_type: DensityOutlineArgType) {
        let selected_frame = if let Some(frame) = self
            .stack
            .get_mut(self.selected_frame.borrow().index as usize)
        {
            frame
        } else {
            return;
        };
        let selected_frame_arg = self.selected_frame.borrow().arg_num;
        selected_frame.arg_types[selected_frame_arg as usize] = arg_type as u8;
        if arg_type == DensityOutlineArgType::Constant {
            selected_frame.arg_positions[selected_frame_arg as usize] =
                self.constant_args.len() as u16;
        } else if arg_type == DensityOutlineArgType::Function {
            selected_frame.arg_positions[selected_frame_arg as usize] =
                self.selected_frame.borrow().largest_slot as u16;
        }
        // Update max slot position
        self.max_slot_position = self
            .max_slot_position
            .max(self.selected_frame.borrow().largest_slot);
    }

    pub fn set_stack_f_arg(&mut self, slot: u8) {
        let selected_frame = if let Some(frame) = self
            .stack
            .get_mut(self.selected_frame.borrow().index as usize)
        {
            frame
        } else {
            return;
        };
        let selected_frame_arg = self.selected_frame.borrow().arg_num;
        selected_frame.arg_types[selected_frame_arg as usize] =
            DensityOutlineArgType::Function as u8;
        selected_frame.arg_positions[selected_frame_arg as usize] = slot as u16;
        // Update max slot position
        self.max_slot_position = self
            .max_slot_position
            .max(self.selected_frame.borrow().largest_slot);
    }

    pub fn push_noise_generator(&mut self, dimension: &str, noise: NoiseArg) {
        let mut hash = AHasher::default();
        noise.get_hash(&mut hash);
        let noise_hash = hash.finish();
        if let Some(pos) = self.noise_generator_map.get(&noise_hash) {
            self.stack[self.selected_frame.borrow().index as usize].noise_index = *pos as u8;
        } else {
            let pos = self.noise_states.len();
            noise.generate_state(dimension, self);
            self.stack[self.selected_frame.borrow().index as usize].noise_index = pos as u8;
            self.noise_generator_map.insert(noise_hash, pos);
        }
    }

    pub fn push_stack_position_as_constant(&mut self) {
        self.constant_args.push(self.stack.len() as f64);
    }

    /// Reserves the nested data counts constants in constant_args
    pub fn push_placeholder_nested_data_counts(&mut self) -> NestedCounts {
        let counts = NestedCounts::new(self.constant_args.len());
        self.constant_args.extend([0.0; 5]);
        counts
    }

    pub fn apply_nested_data_counts(&mut self, counts: NestedCounts) {
        self.constant_args[counts.index] =
            (self.function_flow.len() - counts.function_count) as f64;
        self.constant_args[counts.index + 1] =
            (self.flow_arg_types.len() - counts.argument_count) as f64;
        self.constant_args[counts.index + 2] =
            (self.constant_args.len() - counts.constant_count) as f64;
        self.constant_args[counts.index + 3] =
            (self.noise_states.len() - counts.noise_count) as f64;
        self.constant_args[counts.index + 4] =
            (self.noise_levels.len() - counts.noise_levels_count) as f64;
    }

    // Reserves constants in constant_args for the flow_arg indexes of a functions arguments
    pub fn push_placeholder_argument_positions(&mut self, num_of_args: u8) -> ArgumentPositions {
        let arg_positions = ArgumentPositions::new(self.constant_args.len() as u16);
        self.constant_args.extend(vec![0.0; num_of_args as usize]);
        arg_positions
    }
}

/// Helps with managing the counts data of nested density function arguments for skipping
#[derive(Default, Debug)]
pub struct NestedCounts {
    /// The index in constant_args to place the nested counts data
    index: usize,
    /// Keeps track of constants nested in a density function for skipping
    constant_count: usize,
    /// Keeps track of noise generators nested in a density function for skipping
    noise_count: usize,
    noise_levels_count: usize,
    argument_count: usize,
    /// Keeps track of functions nested in a density function for skipping
    function_count: usize,
}

impl NestedCounts {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            ..Default::default()
        }
    }

    /// Sets the initial data counts of the outline
    pub fn initialize_nested_data_counts(&mut self, outline: &mut DensityFnOutline) {
        self.function_count = outline.function_flow.len();
        self.argument_count = outline.flow_arg_types.len();
        self.constant_count = outline.constant_args.len();
        self.noise_count = outline.noise_states.len();
        self.noise_levels_count = outline.noise_levels.len();
    }
}

#[derive(Default, Debug)]
pub struct ArgumentPositions {
    index: u16,
}

impl ArgumentPositions {
    pub fn new(index: u16) -> Self {
        Self { index }
    }

    pub fn mark_argument_position(&mut self, outline: &mut DensityFnOutline) {
        outline.constant_args[self.index as usize] = outline.flow_arg_types.len() as f64;
        self.index += 1;
    }
}

#[repr(C)]
#[derive(Debug, Zeroable, PartialEq)]
pub enum DensityFnOutlineType {
    Add = 0,
    Mul = 1,
    Min = 2,
    Max = 3,
    Abs = 4,
    Square = 5,
    Cube = 6,
    HalfNegative = 7,
    QuarterNegative = 8,
    Squeeze = 9,
    Clamp = 10,
    YClampedGradient = 11,
    RangeChoice = 12,
    Noise = 13,
    ShiftedNoise = 14,
    Spline = 15,
    WeirdScaledSampler = 16,
    Interpolated = 17,
    BlendDensity = 18,
    BlendOffset = 19,
    BlendAlpha = 20,
    CacheOnce = 21,
    FlatCache = 22,
    Cache2D = 23,
    ShiftA = 24,
    ShiftB = 25,
    OldBlendedNoise = 26,
    EndIslands = 27,
}

#[repr(C)]
#[derive(Debug, Zeroable, PartialEq, Clone, Copy)]
pub enum DensityOutlineArgType {
    Constant = 0,
    Function = 1,
}

#[derive(Deserialize, Debug)]
pub struct BlendOffsetFn;

#[derive(Deserialize, Debug)]
pub struct BlendAlphaFn;

#[derive(Deserialize, Debug)]
pub struct EndIslandsFn;
