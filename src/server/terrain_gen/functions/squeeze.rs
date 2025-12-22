use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SqueezeFn {
    argument: DensityArg,
}

impl SqueezeFn {
    #[inline(always)]
    fn squeeze(value: f64) -> f64 {
        let clamp = value.clamp(-1.0, 1.0);
        (clamp / 2.0) - ((clamp * clamp * clamp) / 24.0)
    }
}

impl DensityFn for SqueezeFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        Self::squeeze(self.argument.compute(args))
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument.compute_slice(args, data);
        for value in data.iter_mut() {
            *value = Self::squeeze(*value);
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        Self::squeeze(self.argument.get_min(args))
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        Self::squeeze(self.argument.get_max(args))
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "squeeze".hash(state);
        self.argument.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::Squeeze);
        self.argument.generate_state(dimension, outline);
    }
}
