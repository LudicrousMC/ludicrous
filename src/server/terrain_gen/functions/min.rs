use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{
    DensityFnOutline, DensityFnOutlineType, DensityOutlineArgType,
};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MinFn {
    argument1: DensityArg,
    argument2: DensityArg,
}

impl DensityFn for MinFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let arg_1 = self.argument1.compute(args);
        if arg_1 < self.argument2.get_min(args) {
            arg_1
        } else {
            arg_1.min(self.argument2.compute(args))
        }
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument1.compute_slice(args, data);
        let arg2_min = self.argument2.get_min(args);
        for (i, value) in data.iter_mut().enumerate() {
            *value = if *value < arg2_min {
                *value
            } else {
                args.mutate_coord_from_slice(i);
                value.min(self.argument2.compute(args))
            };
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument1
            .get_min(args)
            .min(self.argument2.get_min(args))
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument1
            .get_max(args)
            .min(self.argument2.get_max(args))
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "min".hash(state);
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
        // Stack position to go to if fallback min function is not needed
        let next_stack_position = outline.stack.len() as f64;
        // IMPORTANT: Switching or negating this statement will prioritize time over memory usage
        // since it will then use the larger branch's lower bound which reduces computations
        // however the increased memory may make it less efficient overall
        let switch_args =
            self.argument1.get_max_branch_depth() > self.argument2.get_max_branch_depth();

        // Fallback min for argument2 if argument1 is not less than or equal to precomputed argument 2 minimum
        outline.new_stack_frame(DensityFnOutlineType::Min);
        // Indicator constant for fallback min
        outline.constant_args.push(1.0);
        // Generate stack frames for smaller branch and precompute min lower bound
        let arg_min = if switch_args {
            self.argument2.generate_state(dimension, outline);
            self.argument2
                .get_min(&mut DensityFnArgs::new(0, 0, 0, dimension))
        } else {
            self.argument1.generate_state(dimension, outline);
            self.argument1
                .get_min(&mut DensityFnArgs::new(0, 0, 0, dimension))
        };

        // Create primary min function taking into account possible slot usage of fallback min
        outline.new_stack_frame_with_prev_frame(DensityFnOutlineType::Min);
        // Indicator constant for primary min
        outline.constant_args.push(0.0);
        outline.constant_args.push(arg_min);
        // Stack position to go to if fallback function is not needed
        outline.constant_args.push(next_stack_position);
        if switch_args {
            self.argument1.generate_state(dimension, outline);
        } else {
            self.argument2.generate_state(dimension, outline);
        }
    }
}
