#pragma OPENCL EXTENSION cl_khr_fp64 : enable
#include "util.h"
#include "noises.h"
#include "func_state.h"
#include "noise_generator.h"
#include "functions/add.h"
#include "functions/mul.h"
#include "functions/min.h"
#include "functions/max.h"
#include "functions/abs.h"
#include "functions/square.h"
#include "functions/cube.h"
#include "functions/half_negative.h"
#include "functions/quarter_negative.h"
#include "functions/squeeze.h"
#include "functions/clamp.h"
#include "functions/y_clamped_gradient.h"
#include "functions/range_choice.h"
#include "functions/noise.h"
#include "functions/shifted_noise.h"
#include "functions/spline.h"
#include "functions/weird_scaled_sampler.h"
#include "functions/interpolated.h"
#include "functions/blend_density.h"
#include "functions/cache_once.h"
#include "functions/flat_cache.h"
#include "functions/cache_2d.h"
#include "functions/shift_a.h"
#include "functions/shift_b.h"
#include "functions/old_blended_noise.h"

/*
inline double get_stack_arg(DensityFnState* state, uchar arg_num) {
    //FunctionStackFrame frame = state->selected_frame;
    ushort pos = state->stack[state->stack_offset].arg_positions[arg_num];
    return state->stack[state->stack_offset].arg_types[arg_num] == Constant
        ? state->constant_args[pos]
        : state->arg_register[pos];
}

inline void a_compute(DensityFnState* state) {
    state->arg_register[state->stack[state->stack_offset].reg_position] = get_stack_arg(state, 0) + get_stack_arg(state, 1);
}*/

__kernel void eval(
    __constant const FunctionStackFrame* stack,
    short stack_end,
//    __global const DensityArgType* flow_arg_types,
    __global const double* constant_args,
    __constant const VanillaNoise* noise_states,
    __constant const ImprovedNoise* noise_levels,
    __constant const double* noise_amplitudes,
    __global double* out
) {
    int x = get_global_id(0);
    /*int y = get_global_id(1);
    uint size_x = get_global_size(0);
    int z = get_global_id(2);
    uint size_z = get_global_size(2);*/
    DensityFnState state = {
        x, x, x,
        constant_args,
        noise_states,
        noise_levels,
        noise_amplitudes,
        {0},
        stack,
        stack_end
        //0 Selected frame
        //{0}, {0}, 0, 0
    };
    while (state.stack_offset >= 0) {
        DensityFnType fn_type = state.stack[state.stack_offset].fn_type;
        compute_function(fn_type, &state);
        /*for (int i = 0; i < 5; i++) {
            printf("%f ", state.arg_register[i]);
        }
        printf("\n");*/
        state.stack_offset--;
    }
    out[x] = state.arg_register[0];
}

