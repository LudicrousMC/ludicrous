#ifndef OLD_BLENDED_NOISE_H
#define OLD_BLENDED_NOISE_H

inline double old_blended_noise_compute(DensityFnState* state) {
    double smear_scale_multiplier = get_constant_arg(state, 0);
    double xz_factor = get_constant_arg(state, 1);
    double xz_mult = get_constant_arg(state, 2) * 684.412;
    double y_factor = get_constant_arg(state, 3);
    double y_mult = get_constant_arg(state, 4) * 684.412;

    double block_x_mul = state->x * xz_mult;
    double block_y_mul = state->y * y_mult;
    double block_z_mul = state->z * xz_mult;
    double block_x_fact = block_x_mul / xz_factor;
    double block_y_fact = block_y_mul / y_factor;
    double block_z_fact = block_z_mul / xz_factor;
    double y_smear = y_mult * smear_scale_multiplier;
    double y_smear_factor = y_smear / y_factor;
    __constant const VanillaNoise* old_blended_noise = &state->noise_states[0];

    double noise_acc = 0.0;
    double acc = 1.0;
    for (int i = 0; i < 8; i++) {
        __constant const ImprovedNoise* noise_generator = get_perlin_level(&old_blended_noise->noises[1], state, i);
        noise_acc += generate(
            noise_generator,
            wrap(block_x_fact * acc),
            wrap(block_y_fact * acc),
            wrap(block_z_fact * acc),
            y_smear_factor * acc,
            block_y_fact * acc
        ) / acc;
        acc /= 2.0;
    }

    double noise_result = (1.0 + (noise_acc / 10.0)) / 2.0;
    double min_noise_acc = 0.0;
    double max_noise_acc = 0.0;
    acc = 1.0;
    for (int i = 0; i < 16; i++) {
        double block_x_wrap = wrap(block_x_mul * acc);
        double block_y_wrap = wrap(block_y_mul * acc);
        double block_z_wrap = wrap(block_z_mul * acc);
        double y_smear_adj = y_smear * acc;
        double block_y_adj = block_y_mul * acc;
        __constant const ImprovedNoise* noise_limit = get_perlin_level(&old_blended_noise->noises[0], state, i);
        if (!(noise_result >= 1.0)) {
            min_noise_acc += generate(
                noise_limit,
                block_x_wrap,
                block_y_wrap,
                block_z_wrap,
                y_smear_adj,
                block_y_adj
            ) / acc;
        }
        if (!(noise_result <= 0.0)) {
            max_noise_acc += generate(
                noise_limit,
                block_x_wrap,
                block_y_wrap,
                block_z_wrap,
                y_smear_adj,
                block_y_adj
            ) / acc;
        }
        acc /= 2.0;
    }
    if (noise_result < 0.0) {
        return min_noise_acc / (double)(1 << 16);
    } else if (noise_result > 1.0) {
        return max_noise_acc / (double)(1 << 16);
    } else {
        return lerp(
            noise_result,
            min_noise_acc / 512.0,
            max_noise_acc / 512.0
        ) / 128.0;
    }
}

#endif
