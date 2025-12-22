#ifndef HALF_NEGATIVE_H
#define HALF_NEGATIVE_H

inline double half_negative(double value) {
    return value > 0.0 ? value : value * 0.5;
}

inline double half_negative_compute(DensityFnState* state) {
    return half_negative(get_stack_arg(state, 0));
}

#endif
