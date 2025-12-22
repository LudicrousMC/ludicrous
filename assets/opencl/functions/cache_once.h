#ifndef CACHE_ONCE_H
#define CACHE_ONCE_H

inline double cache_once_compute(DensityFnState* state) {
    return get_stack_arg(state, 0);
}

#endif
