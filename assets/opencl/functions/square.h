#ifndef SQUARE_H
#define SQUARE_H

inline double square_compute(DensityFnState* state) {
    double arg = get_stack_arg(state, 0);
    return arg * arg;
}

#endif
