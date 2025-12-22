use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AbsFn {
    argument: DensityArg,
}

impl DensityFn for AbsFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument.compute(args).abs()
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument.compute_slice(args, data);
        for value in data.iter_mut() {
            *value = value.abs();
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        (0.0f64).max(self.argument.get_min(args))
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument
            .get_min(args)
            .abs()
            .max(self.argument.get_max(args).abs())
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "abs".hash(state);
        self.argument.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::Abs);
        self.argument.generate_state(dimension, outline);
    }
}
