#ifndef ADD_H
#define ADD_H

inline double add_compute(DensityFnState* state) {
    return get_stack_arg(state, 0) + get_stack_arg(state, 1);
}

#endif
