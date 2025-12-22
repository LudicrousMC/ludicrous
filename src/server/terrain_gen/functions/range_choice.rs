use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RangeChoiceFn {
    input: DensityArg,
    min_inclusive: f64,
    max_exclusive: f64,
    when_in_range: DensityArg,
    when_out_of_range: DensityArg,
}

impl DensityFn for RangeChoiceFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let input_val = self.input.compute(args);
        if input_val >= self.min_inclusive && input_val < self.max_exclusive {
            self.when_in_range.compute(args)
        } else {
            self.when_out_of_range.compute(args)
        }
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.input.compute_slice(args, data);
        for (i, value) in data.iter_mut().enumerate() {
            args.mutate_coord_from_slice(i);
            *value = if *value >= self.min_inclusive && *value < self.max_exclusive {
                self.when_in_range.compute(args)
            } else {
                self.when_out_of_range.compute(args)
            };
        }
    }

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        self.when_in_range
            .get_min(args)
            .min(self.when_out_of_range.get_min(args))
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.when_in_range
            .get_max(args)
            .max(self.when_out_of_range.get_max(args))
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "range_choice".hash(state);
        self.input.get_tree_hash(state);
        self.min_inclusive.to_be_bytes().hash(state);
        self.max_exclusive.to_be_bytes().hash(state);
        self.when_in_range.get_tree_hash(state);
        self.when_out_of_range.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.input.precompute_noise_instance(dimension);
        self.when_in_range.precompute_noise_instance(dimension);
        self.when_out_of_range.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.input
            .get_max_branch_depth()
            .max(self.when_in_range.get_max_branch_depth())
            .max(self.when_out_of_range.get_max_branch_depth())
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        let next_stack_position = outline.stack.len() as f64;
        let switch_args = self.when_in_range.get_max_branch_depth()
            > self.when_out_of_range.get_max_branch_depth();
        // When in range conditional evaluation:
        outline.new_stack_frame(DensityFnOutlineType::RangeChoice);
        // Indicator constant that this is a conditional of range_choice
        outline.constant_args.push(1.0);
        // Stack position to continue at after range choice is finished
        outline.constant_args.push(next_stack_position);
        if switch_args {
            self.when_out_of_range.generate_state(dimension, outline);
        } else {
            self.when_in_range.generate_state(dimension, outline);
        }
        let in_range_stack_pos = outline.stack.len() as f64;

        // When out of range conditional evaluation:
        outline.new_stack_frame(DensityFnOutlineType::RangeChoice);
        // Indicator constant that this is a conditional of range_choice
        outline.constant_args.push(1.0);
        // Stack position to continue at after range choice is finished
        outline.constant_args.push(next_stack_position);
        if switch_args {
            self.when_in_range.generate_state(dimension, outline);
        } else {
            self.when_out_of_range.generate_state(dimension, outline);
        }
        let out_of_range_stack_pos = outline.stack.len() as f64;

        // Primary range choice evaluation:
        outline.new_stack_frame(DensityFnOutlineType::RangeChoice);
        // Indicator constant that this is the primary range choice function
        outline.constant_args.push(0.0);
        outline.constant_args.push(self.min_inclusive);
        outline.constant_args.push(self.max_exclusive);
        outline.constant_args.push(switch_args as u8 as f64);
        // Stack positions of the in_range conditional and out_of_range conditional
        outline.constant_args.push(in_range_stack_pos);
        outline.constant_args.push(out_of_range_stack_pos);
        self.input.generate_state(dimension, outline);
    }
}
