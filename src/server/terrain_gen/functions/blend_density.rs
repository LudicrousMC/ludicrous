use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};

use super::{DensityArg, DensityFn, DensityFnArgs};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct BlendDensityFn {
    argument: DensityArg,
}

impl DensityFn for BlendDensityFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        // Temporary until blender is implemented in args
        // Will eventually be wrapped in call to blend_density function
        self.argument.compute(args)
    }

    #[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument.compute_slice(args, data);
    }

    #[inline]
    fn get_min(&self, _args: &mut DensityFnArgs) -> f64 {
        unreachable!("Use f64::NEG_INFINITY instead")
    }

    #[inline]
    fn get_max(&self, _args: &mut DensityFnArgs) -> f64 {
        unreachable!("Use f64::INFINITY instead")
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        self.argument.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::BlendDensity);
        self.argument.generate_state(dimension, outline);
    }
}
