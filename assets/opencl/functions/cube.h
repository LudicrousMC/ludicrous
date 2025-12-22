#ifndef CUBE_H
#define CUBE_H

inline double cube(double value) {
    return value * value * value;
}

inline double cube_compute(DensityFnState* state) {
    return cube(get_stack_arg(state, 0));
}

#endif
