use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MulFn {
    argument1: DensityArg,
    argument2: DensityArg,
}

impl DensityFn for MulFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument1.compute(args) * self.argument2.compute(args)
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument1.compute_slice(args, data);
        let mut arg2_data = vec![0f64; data.len()];
        self.argument2.compute_slice(args, &mut arg2_data);
        for (i, value) in data.iter_mut().enumerate() {
            *value *= arg2_data[i];
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        let arg1_min = self.argument1.get_min(args);
        let arg2_min = self.argument2.get_min(args);
        let arg1_max = self.argument1.get_max(args);
        let arg2_max = self.argument2.get_max(args);
        if arg1_min > 0.0 && arg2_min > 0.0 {
            arg1_min * arg2_min
        } else if arg1_max < 0.0 && arg2_max < 0.0 {
            arg1_max * arg2_max
        } else {
            (arg1_min * arg2_max).min(arg1_max * arg2_min)
        }
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        let arg1_min = self.argument1.get_min(args);
        let arg2_min = self.argument2.get_min(args);
        let arg1_max = self.argument1.get_max(args);
        let arg2_max = self.argument2.get_max(args);
        if arg1_min > 0.0 && arg2_min > 0.0 {
            arg1_max * arg2_max
        } else if arg1_max < 0.0 && arg2_max < 0.0 {
            arg1_min * arg2_min
        } else {
            (arg1_min * arg2_min).min(arg1_max * arg2_max)
        }
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "mul".hash(state);
        self.argument1.get_tree_hash(state);
        self.argument2.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument1.precompute_noise_instance(dimension);
        self.argument2.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument1
            .get_max_branch_depth()
            .max(self.argument2.get_max_branch_depth())
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::Mul);

        // Push largest function branch last
        if self.argument1.get_max_branch_depth() > self.argument2.get_max_branch_depth() {
            self.argument2.generate_state(dimension, outline);
            self.argument1.generate_state(dimension, outline);
        } else {
            self.argument1.generate_state(dimension, outline);
            self.argument2.generate_state(dimension, outline);
        }
    }
}
