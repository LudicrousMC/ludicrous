#ifndef RANGE_CHOICE_H
#define RANGE_CHOICE_H

inline double range_choice_compute(DensityFnState* state) {
    int range_choice_type = get_constant_arg(state, 0);
    if (range_choice_type == 0) {
        double min_inclusive = get_constant_arg(state, 1);
        double max_exclusive = get_constant_arg(state, 2);
        uchar switch_args = get_constant_arg(state, 3);
        double input = get_stack_arg(state, 0);
        if (input >= min_inclusive && input < max_exclusive) {
            // Set stack position to in_range conditional function
            state->stack_offset = (int)get_constant_arg(state, switch_args ? 5 : 4);
        } else {
            // Set stack position to out_of_range conditional function
            state->stack_offset = (int)get_constant_arg(state, switch_args ? 4 : 5);
        }
        return 0.0; // Placeholder since this result will be overwritten
    } else {
        double result = get_stack_arg(state, 0);
        state->stack_offset = (int)get_constant_arg(state, 1);
        return result;
    }
}

#endif
