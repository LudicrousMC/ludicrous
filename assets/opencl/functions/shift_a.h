#ifndef SHIFT_A_H
#define SHIFT_A_H

inline double shift_a_compute(DensityFnState* state) {
    return 4.0 * get_vanilla_val(
        state,
        (double)state->x * 0.25,
        0.0,
        (double)state->z * 0.25
    );
}

#endif
