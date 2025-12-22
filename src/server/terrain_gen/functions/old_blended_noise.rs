use std::hash::Hash;

use super::super::super::{randomness::Xoroshiro, util::lerp_f64};
use super::super::noise_generator::PerlinNoise;
use super::super::noise_generator::OLD_BLENDED_NOISE;
use super::{DensityFn, DensityFnArgs};
use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};
use crate::RandomGenerator;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct OldBlendedNoiseFn {
    smear_scale_multiplier: f64,
    xz_factor: f64,
    xz_scale: f64,
    y_factor: f64,
    y_scale: f64,
}

pub struct BlendedNoise {
    pub generator: PerlinNoise,
    pub min_limit: PerlinNoise,
    pub max_limit: PerlinNoise,
}

impl OldBlendedNoiseFn {
    fn create_perlin_noise(rand: &mut Box<dyn RandomGenerator>, first_octave: i32) -> PerlinNoise {
        PerlinNoise::new(
            rand,
            first_octave,
            vec![1.0; -first_octave as usize + 1].into_boxed_slice(),
            false,
        )
    }

    #[inline]
    pub fn create_noise_gen() -> BlendedNoise {
        let mut rand = Xoroshiro::new_from_i64(0);
        let limit = Self::create_perlin_noise(&mut rand, -15);
        BlendedNoise {
            min_limit: limit.clone(),
            max_limit: limit,
            generator: Self::create_perlin_noise(&mut rand, -7),
        }
    }
}

impl DensityFn for OldBlendedNoiseFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let y_mult = self.y_scale * 684.412;
        let xz_mult = self.xz_scale * 684.412;
        let block_x_mul = args.block_x as f64 * xz_mult;
        let block_y_mul = args.block_y as f64 * y_mult;
        let block_z_mul = args.block_z as f64 * xz_mult;
        let block_x_fact = block_x_mul / self.xz_factor;
        let block_y_fact = block_y_mul / self.y_factor;
        let block_z_fact = block_z_mul / self.xz_factor;
        let y_smear = y_mult * self.smear_scale_multiplier;
        let y_smear_factor = y_smear / self.y_factor;

        let mut noise_acc = 0.0;
        let mut acc = 1.0;
        for i in 0..8 {
            if let Some(noise) = OLD_BLENDED_NOISE.generator.get_noise_level(i) {
                noise_acc += noise.generate(
                    PerlinNoise::wrap(block_x_fact * acc),
                    PerlinNoise::wrap(block_y_fact * acc),
                    PerlinNoise::wrap(block_z_fact * acc),
                    y_smear_factor * acc,
                    block_y_fact * acc,
                ) / acc;
            }
            acc /= 2.0;
        }

        let noise_result = (1.0 + (noise_acc / 10.0)) / 2.0;
        let greater_than_zero = noise_result >= 1.0;
        let less_than_or_zero = noise_result <= 0.0;
        let mut min_noise_acc = 0.0;
        let mut max_noise_acc = 0.0;
        acc = 1.0;
        for i in 0..16 {
            let block_x_wrap = PerlinNoise::wrap(block_x_mul * acc);
            let block_y_wrap = PerlinNoise::wrap(block_y_mul * acc);
            let block_z_wrap = PerlinNoise::wrap(block_z_mul * acc);
            let y_smear_adj = y_smear * acc;
            let block_y_adj = block_y_mul * acc;
            if !greater_than_zero {
                if let Some(min_noise) = OLD_BLENDED_NOISE.min_limit.get_noise_level(i) {
                    min_noise_acc += min_noise.generate(
                        block_x_wrap,
                        block_y_wrap,
                        block_z_wrap,
                        y_smear_adj,
                        block_y_adj,
                    ) / acc;
                }
            }

            if !less_than_or_zero {
                if let Some(max_noise) = OLD_BLENDED_NOISE.max_limit.get_noise_level(i) {
                    max_noise_acc += max_noise.generate(
                        block_x_wrap,
                        block_y_wrap,
                        block_z_wrap,
                        y_smear_adj,
                        block_y_adj,
                    ) / acc;
                }
            }
            acc /= 2.0;
        }
        if noise_result < 0.0 {
            min_noise_acc / (1 << 16) as f64
        } else if noise_result > 1.0 {
            max_noise_acc / (1 << 16) as f64
        } else {
            lerp_f64(noise_result, min_noise_acc / 512.0, max_noise_acc / 512.0) / 128.0
        }
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.compute_slice_keep_cache(args, data);
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        -self.get_max(args)
    }

    #[inline]
    fn get_max(&self, _args: &mut DensityFnArgs) -> f64 {
        OLD_BLENDED_NOISE
            .max_limit
            .edge_val(2.0 + (684.412 * self.y_scale))
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "old_blended_noise".hash(state);
        self.smear_scale_multiplier.to_be_bytes().hash(state);
        self.xz_factor.to_be_bytes().hash(state);
        self.xz_scale.to_be_bytes().hash(state);
        self.y_factor.to_be_bytes().hash(state);
        self.y_scale.to_be_bytes().hash(state);
    }

    fn get_max_branch_depth(&self) -> u16 {
        0
    }

    fn generate_state(&self, _dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::OldBlendedNoise);
        outline.constant_args.push(self.smear_scale_multiplier);
        outline.constant_args.push(self.xz_factor);
        outline.constant_args.push(self.xz_scale);
        outline.constant_args.push(self.y_factor);
        outline.constant_args.push(self.y_scale);
    }
}
