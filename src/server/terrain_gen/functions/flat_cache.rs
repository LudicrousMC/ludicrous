use std::{
    cell::RefCell,
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::server::{
    terrain_gen::func_deserialize::{
        CacheFnHelper, DensityFnOutline, DensityFnOutlineType, DensityFnType,
    },
    LudiChunkLoader,
};

use super::{DensityArg, DensityFn, DensityFnArgs};
use ahash::{AHashMap, AHasher};
use serde::Deserialize;

#[derive(Debug)]
pub struct FlatCacheFn {
    argument: DensityArg,
    hash: u64,
}

impl<'de> Deserialize<'de> for FlatCacheFn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let func: CacheFnHelper = Deserialize::deserialize(deserializer)?;
        let mut hasher = AHasher::default();
        func.argument.get_tree_hash(&mut hasher);
        let hash = hasher.finish();
        Ok(FlatCacheFn {
            argument: func.argument,
            hash,
        })
    }
}

impl DensityFn for FlatCacheFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        /*if args.flat_cache_passthrough {
            return self.argument.compute(args);
        }
        let packed_coord = LudiChunkLoader::pack_coords((args.block_x, args.block_z));
        let cache_key = (self.hash as u128) << 64 | packed_coord as u128;
        if args.flat_cache_level != Some(args.block_y as i16) {
            // Clear cache on new y level and compute value
            args.flat_cache.borrow_mut().clear();
            args.flat_cache_level = Some(args.block_y as i16);
            let result = self.argument.compute(args);
            args.flat_cache.borrow_mut().insert(cache_key, result);
            result
        } else {
            let cache_lookup = args.flat_cache.borrow().get(&cache_key).cloned();
            if let Some(result) = cache_lookup {
                // Return cache value if already computed
                result
            } else {
                // Compute value and add to cache if not already computed
                let result = self.argument.compute(args);
                args.flat_cache.borrow_mut().insert(cache_key, result);
                result
            }
        }*/
        self.argument.compute(args)
    }

    /*#[inline]
    fn compute_slice(&self, args: &mut DensityFnArgs, data: &mut Vec<f64>) {
        self.argument.compute_slice(args, data);
    }*/

    #[inline]
    fn get_min(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument.get_min(args)
    }

    #[inline]
    fn get_max(&self, args: &mut DensityFnArgs) -> f64 {
        self.argument.get_max(args)
    }

    fn get_tree_hash(&self, state: &mut ahash::AHasher) {
        // This density function is pure so the hasher is passed through and not mutated
        self.argument.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::FlatCache);
        self.argument.generate_state(dimension, outline);
    }
}
