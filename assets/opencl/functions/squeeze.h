#ifndef SQUEEZE_H
#define SQUEEZE_H

inline double squeeze(double value) {
    double clamped_val = clamp(value, -1.0, 1.0);
    return (clamped_val / 2.0) - ((clamped_val * clamped_val * clamped_val) / 24.0);
}

inline double squeeze_compute(DensityFnState* state) {
    return squeeze(get_stack_arg(state, 0));
}

#endif
