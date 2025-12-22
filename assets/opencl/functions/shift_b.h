#ifndef SHIFT_B_H
#define SHIFT_B_H

inline double shift_b_compute(DensityFnState* state) {
    return 4.0 * get_vanilla_val(
        state,
        (double)state->z * 0.25,
        (double)state->x * 0.25,
        0.0
    );
}

#endif
