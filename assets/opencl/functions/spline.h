#ifndef SPLINE_H
#define SPLINE_H

inline double linear_ext_if_non_zero(double x, double derivative, double location, double value) {
    return derivative == 0.0
        ? value
        : value + (derivative * (x - location));
}

inline double spline_value_compute(DensityFnState* state) {
    int point_count = get_constant_arg(state, 1);
    double coordinate_val = get_stack_arg(state, 0);
    int search_len = point_count;
    int search_index = 0;
    while (search_len > 0) {
        int half_len = search_len / 2;
        int mid = search_index + half_len;
        double point_location = get_constant_arg(state, mid + 2);
        if (coordinate_val < point_location) {
            search_len = half_len;
        } else {
            search_index = mid + 1;
            search_len -= half_len + 1;
        }
    }
    search_index -= 1;
    double result;
    if (search_index < 0) {
        //SplinePoint first_point = get_spline_point(state, 0);
        //result = linear_ext_if_non_zero(first_point, coordinate_val, first_point.value);
    } else if (search_index == point_count - 1) {
        
        //SplinePoint last_point = get_spline_point(state, point_count - 1);
        //result = linear_ext_if_non_zero(last_point, coordinate_val, last_point.value);
    } else {
        /*SplinePoint point = get_spline_point(state, search_index);
        SplinePoint next_point = get_spline_point(state, search_index + 1);
        double point_distance = next_point.location - point.location;
        double position = (coordinate_val - point.location) / point_distance;
        double point_value_distance = next_point.value - point.value;
        double val1 = point.derivative * point_distance - point_value_distance;
        double val2 = -next_point.derivative * point_distance + point_value_distance;*/
        result = 0.0;

        /*result = lerp(position, point.value, next_point.value)
            + (position * (1.0 - position)) * lerp(position, val1, val2);*/
    }
    return result;
}

inline double spline_compute(DensityFnState* state) {
    int spline_type = get_constant_arg(state, 0);
    //printf("spline\n");
    if (spline_type == 0) {
        //printf("primary\n");
        double coordinate_val = get_stack_arg(state, 0);
        int point_count = get_constant_arg(state, 1);
        // search index and len is offset by 3 to avoid adding 3 when searching
        int search_len = point_count;
        int search_index = 0;
        while (search_len > 0) {
            int half_len = search_len / 2;
            int mid = search_index + half_len;
            double point_location = get_constant_arg(state, mid + 3);
            if (coordinate_val < point_location) {
                search_len = half_len;
            } else {
                search_index = mid + 1;
                search_len -= half_len + 1;
            }
        }
        search_index -= 1;
        state->skip_result = 1;
        if (search_index < 0) {
            // The + 2 is the offset due to coord val and point count
            // The (point_count * 2) is the offset from the location and derivatives
            int first_point_stack = get_constant_arg(state, (point_count * 2) + 3);
            state->arg_register[state->stack[state->stack_offset].reg_position] = 0.0;
            // Hand calculations over to the calculated point
            state->stack_offset = first_point_stack;
        } else if (search_index == point_count - 1) {
            // The + 1 is the offset due to coord val and point count minus 1 due to point_count * 3 being the length
            // The (point_count * 3) is the offset from the location and derivatives
            int point_stack = get_constant_arg(state, (point_count * 3) + 2);
            state->arg_register[state->stack[state->stack_offset].reg_position] = (double)(search_index + 1);
            // Hand calculations over to the calculated point
            state->stack_offset = point_stack;
            //printf("%d\n", state->stack[state->stack_offset].reg_position);
        } else {
            // Get point stack location after the point at the search index
            int point_stack = get_constant_arg(state, (point_count * 2) + 4 + search_index);
            state->arg_register[state->stack[state->stack_offset].reg_position] = (double)search_index + 1.0;
            state->stack_offset = point_stack;
        }
        return 0.0;
    } else if (spline_type == 1) {
      //printf("point\n");
        int old_stack_offset = state->stack_offset;
        double value = get_stack_arg(state, 0);
        // Change stack frame context to spline primary frame
        int spline_stack_pos = get_constant_arg(state, 1);
        state->stack_offset = spline_stack_pos;
        int point_num = state->arg_register[state->stack[state->stack_offset].reg_position];
        double coord = get_stack_arg(state, 0);
        int num_of_points = get_constant_arg(state, 1);
        int init_stack_pos = get_constant_arg(state, 2);
        //printf("%d %d\n", point_num, num_of_points);
        //state->stack_offset = old_stack_offset;
        //printf("%d\n", point_num);
        if (point_num == 0) {
            double point_deriv = get_constant_arg(state, num_of_points + 3 + point_num);
            double point_location = get_constant_arg(state, 3 + point_num);
            // Return value of point if this is the first or last point
            // Change stack position to before spline
            state->arg_register[state->stack[state->stack_offset].reg_position] = 
                linear_ext_if_non_zero(coord, point_deriv, point_location, value);
            state->stack_offset = init_stack_pos;
            state->skip_result = 1;
            return 0.0;
        } else if (point_num == num_of_points) {
            double point_deriv = get_constant_arg(state, num_of_points + 2 + point_num);
            double point_location = get_constant_arg(state, 2 + point_num);
            // Return value of point if this is the first or last point
            // Change stack position to before spline
            //printf("%f \n", value);
            state->arg_register[state->stack[state->stack_offset].reg_position] =
                linear_ext_if_non_zero(coord, point_deriv, point_location, value);
            state->stack_offset = init_stack_pos;
            state->skip_result = 1;
            return 0.0;
            //return linear_ext_if_non_zero(coord, point_deriv, point_location, value);
        } else {
            // Get the stack location of the point before the final and adds one to
            // get the stack location of the final point
            double point_deriv = get_constant_arg(state, num_of_points + 2 + point_num);
            double point_location = get_constant_arg(state, 2 + point_num);
            int final_point_stack = get_constant_arg(state, (num_of_points * 2) + 3 + (point_num - 1));
            if (old_stack_offset != final_point_stack) {
                double next_point_deriv = get_constant_arg(state, num_of_points + 3 + point_num);
                double next_point_location = get_constant_arg(state, 3 + point_num);
                int next_point_stack = get_constant_arg(state, (num_of_points * 2) + 2 + point_num);
                state->stack_offset = next_point_stack;

                double next_point_value = get_stack_arg(state, 0);
                double distance = next_point_location - point_location;
                double position = (coord - point_location) / distance;
                double value_distance = next_point_value - value;
                double val1 = point_deriv * distance - value_distance;
                double val2 = -next_point_deriv * distance + value_distance;
                //printf("%f %f %f\n", point_deriv, point_location, value);
                //printf("%f %f %f\n", next_point_deriv, next_point_location, next_point_value);
                double new_val = 
                    lerp(position, value, next_point_value)
                        + (position * (1.0 - position)) * lerp(position, val1, val2);
                state->arg_register[state->stack[spline_stack_pos].reg_position] = new_val;
                //printf("%f \n", new_val);
                state->stack_offset = init_stack_pos;
                state->skip_result = 1;
                //return state->arg_register[state->stack[spline_stack_pos].reg_position];
                return new_val;
            } else {
                // Keep current stack position so that the point before is computed
                state->stack_offset = old_stack_offset;
                return linear_ext_if_non_zero(coord, point_deriv, point_location, value);
            }
        }
        
    } else if (spline_type == 2) {
        return get_stack_arg(state, 0);
    }
}

#endif
