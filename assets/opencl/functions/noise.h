#ifndef NOISE_H
#define NOISE_H

inline double noise_compute(DensityFnState* state) {
    double xz_scale = get_constant_arg(state, 0);
    double y_scale = get_constant_arg(state, 1);
    return get_vanilla_val(
        state,
        (double)state->x * xz_scale,
        (double)state->y * y_scale,
        (double)state->z * xz_scale
    );
}

#endif
