use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ClampFn {
    input: DensityArg,
    min: f64,
    max: f64,
}

impl ClampFn {
    #[inline]
    pub fn get_min_no_args(&self) -> f64 {
        self.min
    }

    #[inline]
    pub fn get_max_no_args(&self) -> f64 {
        self.max
    }
}

impl DensityFn for ClampFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        self.input.compute(args).clamp(self.min, self.max)
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.input.compute_slice(args, data);
        for value in data.iter_mut() {
            *value = (*value).clamp(self.min, self.max);
        }
    }

    fn get_min(&self, _args: &mut DensityFnArgs) -> f64 {
        unreachable!("Use get_min_no_args() instead")
    }

    fn get_max(&self, _args: &mut DensityFnArgs) -> f64 {
        unreachable!("Use get_max_no_args() instead")
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "clamp".hash(state);
        self.input.get_tree_hash(state);
        self.max.to_be_bytes().hash(state);
        self.min.to_be_bytes().hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.input.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.input.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::Clamp);
        outline.constant_args.push(self.min);
        outline.constant_args.push(self.max);
        self.input.generate_state(dimension, outline);
    }
}
