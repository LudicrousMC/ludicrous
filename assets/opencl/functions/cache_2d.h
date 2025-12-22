#ifndef CACHE_2D_H
#define CACHE_2D_H

inline double cache_2d_compute(DensityFnState* state) {
    return get_stack_arg(state, 0);
}

#endif
