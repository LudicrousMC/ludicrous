use std::hash::Hash;

use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::super::super::util::clamped_map_f64;
use super::{DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct YClampedGradientFn {
    from_y: i32, // i32
    to_y: i32,   // i32
    from_value: f64,
    to_value: f64,
}

impl YClampedGradientFn {
    #[inline]
    pub fn get_min_no_args(&self) -> f64 {
        self.from_value.min(self.to_value)
    }

    #[inline]
    pub fn get_max_no_args(&self) -> f64 {
        self.from_value.max(self.to_value)
    }
}

impl DensityFn for YClampedGradientFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        clamped_map_f64(
            args.block_y as f64,
            self.from_y as f64,
            self.to_y as f64,
            self.from_value,
            self.to_value,
        )
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.compute_slice_keep_cache(args, data);
    }

    #[inline]
    fn get_min(&self, _args: &mut DensityFnArgs) -> f64 {
        unreachable!("Use get_min_no_args() instead")
    }

    #[inline]
    fn get_max(&self, _args: &mut DensityFnArgs) -> f64 {
        unreachable!("Use get_max_no_args() instead")
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "y_clamped_gradient".hash(state);
        self.from_y.to_be_bytes().hash(state);
        self.to_y.to_be_bytes().hash(state);
        self.from_value.to_be_bytes().hash(state);
        self.to_value.to_be_bytes().hash(state);
    }

    fn get_max_branch_depth(&self) -> u16 {
        0
    }

    fn generate_state(&self, _dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::YClampedGradient);
        outline.constant_args.push(self.from_y as f64);
        outline.constant_args.push(self.to_y as f64);
        outline.constant_args.push(self.from_value);
        outline.constant_args.push(self.to_value);
    }
}
