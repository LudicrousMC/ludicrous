use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs, NoiseArg};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "rarity_value_mapper")]
pub enum WeirdScaledSamplerFn {
    #[serde(rename = "type_1")]
    Type1(WeirdScaledSamplerType),
    #[serde(rename = "type_2")]
    Type2(WeirdScaledSamplerType),
}

impl WeirdScaledSamplerFn {
    fn get_sampler(&self) -> &WeirdScaledSamplerType {
        match self {
            Self::Type1(sampler) => sampler,
            Self::Type2(sampler) => sampler,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct WeirdScaledSamplerType {
    input: DensityArg,
    noise: NoiseArg,
}

impl DensityFn for WeirdScaledSamplerFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let (sampler, rarity) = match self {
            Self::Type1(sampler1) => {
                let input_val = sampler1.input.compute(args);
                (sampler1, spaghetti_rarity_3d(input_val))
            }
            Self::Type2(sampler2) => {
                let input_val = sampler2.input.compute(args);
                (sampler2, spaghetti_rarity_2d(input_val))
            }
        };
        rarity
            * sampler
                .noise
                .get_or_create(args.dimension)
                .get_val(
                    args.block_x as f64 / rarity,
                    args.block_y as f64 / rarity,
                    args.block_z as f64 / rarity,
                )
                .abs()
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        let (sampler, rarity_fn) = match self {
            Self::Type1(sampler1) => {
                sampler1.input.compute_slice(args, data);
                (sampler1, spaghetti_rarity_3d as fn(f64) -> f64)
            }
            Self::Type2(sampler2) => {
                sampler2.input.compute_slice(args, data);
                (sampler2, spaghetti_rarity_2d as fn(f64) -> f64)
            }
        };
        let noise = sampler.noise.get_or_create(args.dimension);
        for (i, value) in data.iter_mut().enumerate() {
            let rarity = rarity_fn(*value);
            args.mutate_coord_from_slice(i);
            *value = rarity
                * noise
                    .get_val(
                        args.block_x as f64 / rarity,
                        args.block_y as f64 / rarity,
                        args.block_z as f64 / rarity,
                    )
                    .abs();
        }
    }

    #[inline]
    fn get_min(&self, _args: &mut DensityFnArgs) -> f64 {
        0.0
    }

    #[inline]
    fn get_max(&self, _args: &mut DensityFnArgs) -> f64 {
        match self {
            Self::Type1(_) => 2.0,
            Self::Type2(_) => 3.0,
        }
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "weird_scaled_sampler".hash(state);
        let sampler = match self {
            Self::Type1(sampler) => {
                "type_1".hash(state);
                sampler
            }
            Self::Type2(sampler) => {
                "type_2".hash(state);
                sampler
            }
        };
        sampler.input.get_tree_hash(state);
        sampler.noise.get_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        let sampler = match self {
            Self::Type1(sampler) => sampler,
            Self::Type2(sampler) => sampler,
        };
        sampler.noise.precompute_noise_instance(dimension);
        sampler.input.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.get_sampler().input.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::WeirdScaledSampler);
        let sampler_type = match self {
            Self::Type1(_) => 0.0,
            Self::Type2(_) => 1.0,
        };
        outline.constant_args.push(sampler_type);
        let sampler = self.get_sampler();
        outline.push_noise_generator(dimension, sampler.noise.clone());
        sampler.input.generate_state(dimension, outline);
    }
}

#[inline(always)]
fn spaghetti_rarity_3d(value: f64) -> f64 {
    if value < -0.5 {
        0.75
    } else if value < 0.0 {
        1.0
    } else if value < 0.5 {
        1.5
    } else {
        2.0
    }
}

#[inline(always)]
fn spaghetti_rarity_2d(value: f64) -> f64 {
    if value < -0.75 {
        0.5
    } else if value < -0.5 {
        0.75
    } else if value < 0.5 {
        1.0
    } else if value < 0.75 {
        2.0
    } else {
        3.0
    }
}
