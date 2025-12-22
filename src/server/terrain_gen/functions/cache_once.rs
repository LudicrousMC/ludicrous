use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::server::terrain_gen::func_deserialize::{
    CacheFnHelper, DensityFnOutline, DensityFnOutlineType, DensityFnType,
};

use super::super::super::chunk_system::LudiChunkLoader;
use super::{DensityArg, DensityFn, DensityFnArgs};
use ahash::{AHashMap, AHasher};
use serde::Deserialize;

#[derive(Debug)]
pub struct CacheOnceFn {
    argument: DensityArg,
    hash: u64,
}

impl<'de> Deserialize<'de> for CacheOnceFn {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let func: CacheFnHelper = Deserialize::deserialize(deserializer)?;
        let mut hasher = AHasher::default();
        func.argument.get_tree_hash(&mut hasher);
        let hash = hasher.finish();
        Ok(CacheOnceFn {
            argument: func.argument,
            hash,
        })
    }
}

impl DensityFn for CacheOnceFn {
    #[inline]
    fn compute(&self, args: &mut DensityFnArgs) -> f64 {
        let packed_coord = LudiChunkLoader::pack_xyz((args.block_x, args.block_y, args.block_z));
        /*if args.once_cache_pos != Some(packed_coord) {
            args.once_cache.borrow_mut().clear();
            let result = self.argument.compute(args);
            args.once_cache_pos = Some(packed_coord);
            args.once_cache.borrow_mut().insert(self.hash, result);
            result
        } else {
            let cache_lookup = args.once_cache.borrow().get(&self.hash).cloned();
            if let Some(result) = cache_lookup {
                result
            } else {
                let result = self.argument.compute(args);
                args.once_cache.borrow_mut().insert(self.hash, result);
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
        "cache_once".hash(state);
        self.argument.get_tree_hash(state);
    }

    fn precompute_noise_instance(&self, dimension: &str) {
        self.argument.precompute_noise_instance(dimension);
    }

    fn get_max_branch_depth(&self) -> u16 {
        self.argument.get_max_branch_depth()
    }

    fn generate_state(&self, dimension: &str, outline: &mut DensityFnOutline) {
        outline.new_stack_frame(DensityFnOutlineType::CacheOnce);
        self.argument.generate_state(dimension, outline);
    }
}
