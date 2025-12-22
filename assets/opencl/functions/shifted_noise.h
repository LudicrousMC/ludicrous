#ifndef SHIFTED_NOISE_H
#define SHIFTED_NOISE_H

inline double shifted_noise_compute(DensityFnState* state) {
    double xz_scale = get_constant_arg(state, 0);
    double y_scale = get_constant_arg(state, 1);
    // Get order of corrdinate offset arguments
    uchar x_arg = get_constant_arg(state, 2);
    uchar y_arg = get_constant_arg(state, 3);
    uchar z_arg = get_constant_arg(state, 4);
    double shifted_x = ((double)state->x * xz_scale) + get_stack_arg(state, x_arg);
    double shifted_y = ((double)state->y * y_scale) + get_stack_arg(state, y_arg);
    double shifted_z = ((double)state->z * xz_scale) + get_stack_arg(state, z_arg);
    return get_vanilla_val(state, shifted_x, shifted_y, shifted_z);
}

#endif
