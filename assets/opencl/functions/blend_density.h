#ifndef BLEND_DENSITY_H
#define BLEND_DENSITY_H

inline double blend_density_compute(DensityFnState* state) {
    return get_stack_arg(state, 0);
}

#endif
