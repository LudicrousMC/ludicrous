#ifndef Y_CLAMPED_GRADIENT_H
#define Y_CLAMPED_GRADIENT_H

inline double y_clamped_gradient_compute(DensityFnState* state) {
    double from_y = get_constant_arg(state, 0);
    double to_y = get_constant_arg(state, 1);
    double from_value = get_constant_arg(state, 2);
    double to_value = get_constant_arg(state, 3);
    return clamped_map(
        (double)state->y,
        from_y,
        to_y,
        from_value,
        to_value
    );
}

#endif
