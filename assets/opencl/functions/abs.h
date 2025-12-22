#ifndef ABS_H
#define ABS_H

inline double abs_compute(DensityFnState* state) {
    return fabs(get_stack_arg(state, 0));
}

#endif
