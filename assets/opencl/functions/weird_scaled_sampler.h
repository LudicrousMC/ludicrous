#ifndef WEIRD_SCALED_SAMPLER_H
#define WEIRD_SCALED_SAMPLER_H

inline double spaghetti_rarity_3d(double value) {
    return value < -0.5
        ? 0.75
        : value < 0.0
        ? 1.0
        : value < 0.5
        ? 1.5
        : 2.0;
}

inline double spaghetti_rarity_2d(double value) {
    return value < -0.75
        ? 0.5
        : value < -0.5
        ? 0.75
        : value < 0.5
        ? 1.0
        : value < 0.75
        ? 2.0
        : 3.0;
}

inline double weird_scaled_sampler_compute(DensityFnState* state) {
    int mapper_type = get_constant_arg(state, 0);
    double input_val = get_stack_arg(state, 0);
    double rarity;
    if (mapper_type == 0) {
        rarity = spaghetti_rarity_3d(input_val);
    } else if (mapper_type == 1) {
        rarity = spaghetti_rarity_2d(input_val);
    }
    return rarity * fabs(get_vanilla_val(
        state,
        (double)state->x / rarity,
        (double)state->y / rarity,
        (double)state->z / rarity
    ));
}

#endif
