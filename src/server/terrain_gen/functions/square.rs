use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SquareFn {
    argument: DensityArg,
}

impl DensityFn for SquareFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let arg_val = self.argument.compute(args);
        arg_val * arg_val
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument.compute_slice(args, data);
        for value in data.iter_mut() {
            *value *= *value;
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        (0.0f64).max(self.argument.get_min(args))
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        let arg_min = self.argument.get_min(args);
        let arg_max = self.argument.get_max(args);
        (arg_min * arg_min).max(arg_max * arg_max)
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "square".hash(state);
        self.argument.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::Square);
        self.argument.generate_state(dimension, outline);
    }
}
