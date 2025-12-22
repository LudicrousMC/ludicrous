#ifndef QUARTER_NEGATIVE_H
#define QUARTER_NEGATIVE_H

inline double quarter_negative(double value) {
    return value > 0.0 ? value : value * 0.25;
}

inline double quarter_negative_compute(DensityFnState* state) {
    return quarter_negative(get_stack_arg(state, 0));
}

#endif
