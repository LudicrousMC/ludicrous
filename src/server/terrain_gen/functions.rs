mod abs;
mod add;
mod blend_density;
mod cache_2d;
mod cache_once;
mod clamp;
mod cube;
mod flat_cache;
mod half_negative;
mod interpolated;
mod max;
mod min;
mod mul;
mod noise;
mod old_blended_noise;
mod quarter_negative;
mod range_choice;
mod shift_a;
mod shift_b;
mod shifted_noise;
mod spline;
mod square;
mod squeeze;
mod weird_scaled_sampler;
mod y_clamped_gradient;
use crate::server::terrain_gen::func_deserialize::DensityFnOutline;

use super::func_deserialize::{DensityArg, DensityFnArgs, NoiseArg};
pub use abs::AbsFn;
pub use add::AddFn;
use ahash::AHasher;
pub use blend_density::BlendDensityFn;
pub use cache_2d::Cache2DFn;
pub use cache_once::CacheOnceFn;
pub use clamp::ClampFn;
pub use cube::CubeFn;
pub use flat_cache::FlatCacheFn;
pub use half_negative::HalfNegativeFn;
pub use interpolated::InterpolatedFn;
pub use max::MaxFn;
pub use min::MinFn;
pub use mul::MulFn;
pub use noise::NoiseFn;
pub use old_blended_noise::{BlendedNoise, OldBlendedNoiseFn};
pub use quarter_negative::QuarterNegativeFn;
pub use range_choice::RangeChoiceFn;
pub use shift_a::ShiftAFn;
pub use shift_b::ShiftBFn;
pub use shifted_noise::ShiftedNoiseFn;
pub use spline::SplineFn;
pub use square::SquareFn;
pub use squeeze::SqueezeFn;
pub use weird_scaled_sampler::WeirdScaledSamplerFn;
pub use y_clamped_gradient::YClampedGradientFn;

pub trait DensityFn {
    fn compute(&self, args: &mut DensityFnArgs) -> f64;
    /// Computes each value in a data array individually. This is inefficient for certain density
    /// function implementations, in which case it should be overridden
    /// Resets cache when passing args to children
    #[inline(always)]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        for (i, value) in data.iter_mut().enumerate() {
            args.mutate_coord_from_slice(i);
            *value = self.compute(args);
        }
    }
    #[inline(always)]
    fn compute_slice_keep_cache(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        for (i, value) in data.iter_mut().enumerate() {
            args.mutate_coord_from_slice(i);
            *value = self.compute(args);
        }
    }
    fn get_min(&self, args: &mut DensityFnArgs) -> f64;
    fn get_max(&self, args: &mut DensityFnArgs) -> f64;
    fn get_tree_hash(&self, state: &mut AHasher);
    fn precompute_noise_instance(&self, dimension: &str) {}
    fn get_max_branch_depth(&self) -> u16 {
        todo!()
    }
    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {}
}
