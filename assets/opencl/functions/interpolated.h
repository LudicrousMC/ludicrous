#ifndef INTERPOLATED_H
#define INTERPOLATED_H

inline double interpolated_compute(DensityFnState* state) {
    return get_stack_arg(state, 0);
}

#endif
