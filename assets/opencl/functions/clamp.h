#ifndef CLAMP_H
#define CLAMP_H

inline double clamp_compute(DensityFnState* state) {
    double min = get_constant_arg(state, 0);
    double max = get_constant_arg(state, 1);
    double input = get_stack_arg(state, 0);
    return clamp(input, min, max);
}

#endif
