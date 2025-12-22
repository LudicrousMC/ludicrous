#ifndef FLAT_CACHE_H
#define FLAT_CACHE_H

inline double flat_cache_compute(DensityFnState* state) {
    return get_stack_arg(state, 0);
}

#endif
