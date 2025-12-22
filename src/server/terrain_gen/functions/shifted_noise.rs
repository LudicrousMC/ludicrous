use std::hash::Hash;

use crate::server::{
    terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType},
    LudiChunkLoader,
};

use super::{DensityArg, DensityFn, DensityFnArgs, NoiseArg};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ShiftedNoiseFn {
    noise: NoiseArg,
    xz_scale: f64,
    y_scale: f64,
    shift_x: DensityArg,
    shift_y: DensityArg,
    shift_z: DensityArg,
}

impl DensityFn for ShiftedNoiseFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let shifted_x = (args.block_x as f64 * self.xz_scale) + self.shift_x.compute(args);
        let shifted_y = (args.block_y as f64 * self.y_scale) + self.shift_y.compute(args);
        let shifted_z = (args.block_z as f64 * self.xz_scale) + self.shift_z.compute(args);
        self.noise
            .get_or_create(args.dimension)
            .get_val(shifted_x, shifted_y, shifted_z)
    }

    /*#[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        let mut positions = vec![(0.0, 0.0, 0.0); args.slice_positions.len()];
        for (i, pos) in args.slice_positions.iter().enumerate() {
            let (x, y, z) = LudiChunkLoader::unpack_xyz(*pos);
            positions[i] = (
                (x as f64 * self.xz_scale) + self.shift_x.compute(args),
                (y as f64 * self.y_scale) + self.shift_y.compute(args),
                (z as f64 * self.xz_scale) + self.shift_z.compute(args),
            );
        }
        let results = self.noise.get_or_create(args.dimension).get_val_batch(&positions);
        for (i, value) in results.into_iter().enumerate(){
            data[i] = value;
        }
        //self.compute_slice_keep_cache(args, data);
    }*/

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        -self.get_max(args)
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.noise.get_or_create(args.dimension).get_max()
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        "shifted_noise".hash(state);
        self.noise.get_hash(state);
        self.xz_scale.to_be_bytes().hash(state);
        self.y_scale.to_be_bytes().hash(state);
        self.shift_x.get_tree_hash(state);
        self.shift_y.get_tree_hash(state);
        self.shift_z.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.noise.precompute_noise_instance(dimension);
        self.shift_x.precompute_noise_instance(dimension);
        self.shift_y.precompute_noise_instance(dimension);
        self.shift_z.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.shift_x
            .get_max_branch_depth()
            .max(self.shift_y.get_max_branch_depth())
            .max(self.shift_z.get_max_branch_depth())
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::ShiftedNoise);
        outline.constant_args.push(self.xz_scale);
        outline.constant_args.push(self.y_scale);
        let const_index = outline.constant_args.len();
        outline.constant_args.extend([0.0; 3]);
        outline.push_noise_generator(dimension, self.noise.clone());
        let mut args = [
            (self.shift_x.get_max_branch_depth(), 0, &self.shift_x),
            (self.shift_y.get_max_branch_depth(), 1, &self.shift_y),
            (self.shift_z.get_max_branch_depth(), 2, &self.shift_z),
        ];

        args.sort_by_key(|x| x.0);
        for (i, a) in args.iter().enumerate() {
            outline.constant_args[const_index + a.1] = i as f64;
            a.2.generate_state(dimension, outline);
        }
    }
}
