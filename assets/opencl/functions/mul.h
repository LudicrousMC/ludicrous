#ifndef MUL_H
#define MUL_H

inline double mul_compute(DensityFnState* state) {
    return get_stack_arg(state, 0) * get_stack_arg(state, 1);
}

#endif
