use crate::server::terrain_gen::func_deserialize::{DensityFnOutline, DensityFnOutlineType};
use std::hash::Hash;

use super::{DensityFn, DensityFnArgs, NoiseArg};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct NoiseFn {
    noise: NoiseArg,
    xz_scale: f64,
    y_scale: f64,
}

impl DensityFn for NoiseFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        self.noise.get_or_create(args.dimension).get_val(
            args.block_x as f64 * self.xz_scale,
            args.block_y as f64 * self.y_scale,
            args.block_z as f64 * self.xz_scale,
        )
    }

    /*#[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        //let time = std::time::Instant::now();
        let mut positions = vec![(0.0, 0.0, 0.0); args.slice_positions.len()];
        for (i, pos) in args.slice_positions.iter().enumerate() {
            let (x, y, z) = LudiChunkLoader::unpack_xyz(*pos);
            positions[i] = (
                x as f64 * self.xz_scale,
                y as f64 * self.y_scale,
                z as f64 * self.xz_scale,
            );
        }
        let results = self.noise.get_or_create(args.dimension).get_val_batch(&positions);
        for (i, value) in results.into_iter().enumerate(){
            data[i] = value;
        }
        //println!("noise time {:?}", time.elapsed());
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
        "noise".hash(state);
        self.noise.get_hash(state);
        self.xz_scale.to_be_bytes().hash(state);
        self.y_scale.to_be_bytes().hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.noise.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        0
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::Noise);
        outline.constant_args.push(self.xz_scale);
        outline.constant_args.push(self.y_scale);
        outline.push_noise_generator(dimension, self.noise.clone());
    }
}
