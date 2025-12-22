#ifndef MIN_H
#define MIN_H

inline double min_compute(DensityFnState* state) {
    short min_type = get_constant_arg(state, 0);
    // Calculate min based on min function type which is whether to use precomputed arg2 lower bound or
    // computed arg2 result. 0 = Primary (use arg2 lower bound), 1 = Fallback (compute arg2)
    if (min_type == 0) {
        double arg1 = get_stack_arg(state, 0);
        double arg2 = get_constant_arg(state, 1);
        // If arg1 is less than or equal to precomputed arg2 lower bound then skip fallback
        if (arg1 <= arg2) {
            short next_stack_pos = get_constant_arg(state, 2);
            // Store result in the same register as the fallback
            state->arg_register[state->stack[next_stack_pos].reg_position] = arg1;
            // Skip fallback
            state->stack_offset = next_stack_pos;
        }
        return arg1;
    } else {
        // Fallback min
        double arg1 = get_stack_arg(state, 0);
        double arg2 = get_stack_arg(state, 1);
        return arg1 < arg2 ? arg1 : arg2;
    }
}

#endif
