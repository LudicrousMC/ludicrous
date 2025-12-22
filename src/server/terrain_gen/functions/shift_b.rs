use std::hash::Hash;

use crate::server::{
    terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType},
    LudiChunkLoader,
};

use super::{DensityFn, DensityFnArgs, NoiseArg};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ShiftBFn {
    argument: NoiseArg,
}

impl DensityFn for ShiftBFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        4.0 * self.argument.get_or_create(args.dimension).get_val(
            args.block_z as f64 * 0.25,
            args.block_x as f64 * 0.25,
            0.0,
        )
    }

    /*#[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        let mut positions = vec![(0.0, 0.0, 0.0); args.slice_positions.len()];
        for (i, pos) in args.slice_positions.iter().enumerate() {
            let (x, _y, z) = LudiChunkLoader::unpack_xyz(*pos);
            positions[i] = (
                z as f64 * 0.25,
                x as f64 * 0.25,
                0.0,
            );
        }
        let results = self.argument.get_or_create(args.dimension).get_val_batch(&positions);
        for (i, value) in results.into_iter().enumerate(){
            data[i] = 4.0 * value;
        }
        //self.compute_slice_keep_cache(args, data);
    }*/

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        -self.get_max(args)
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        4.0 * self.argument.get_or_create(args.dimension).get_max()
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "shift_b".hash(state);
        self.argument.get_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        0
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::ShiftB);
        outline.push_noise_generator(dimension, self.argument.clone());
    }
}
