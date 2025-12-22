use super::{DensityArg, DensityFn, DensityFnArgs};
use crate::server::{
    terrain_gen::func_deserialize::{CacheFnHelper, DensityFnOutline, DensityFnOutlineType},
    util::lerp_f64,
    LudiChunkLoader,
};
use ahash::AHasher;
use serde::Deserialize;
use std::hash::Hasher;

#[derive(Debug)]
pub struct InterpolatedFn {
    argument: DensityArg,
    hash: u64,
}

impl<'de> Deserialize<'de> for InterpolatedFn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let func: CacheFnHelper = Deserialize::deserialize(deserializer)?;
        let mut hasher = AHasher::default();
        func.argument.get_tree_hash(&mut hasher);
        let hash = hasher.finish();
        Ok(InterpolatedFn {
            argument: func.argument,
            hash,
        })
    }
}

impl DensityFn for InterpolatedFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        // Interpolation cache is already implemented at the chunk level
        self.argument.compute(args)
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument.compute_slice(args, data);
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument.get_min(args)
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument.get_max(args)
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        // This density function is pure so the hasher is passed through and not mutated
        self.argument.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        // Might be able to eliminate these cache type stack frames by adding a frame flag to the
        // inner argument that signals to cache the function result thereby reducing stack size.
        // However, that might add extra operations since it has to check each frame for the flag
        // so implementing this needs a/b timing tests
        outline.new_stack_frame(DensityFnOutlineType::Interpolated);
        self.argument.generate_state(dimension, outline);
    }
}
