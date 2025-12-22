#ifndef FUNC_STATE_H
#define FUNC_STATE_H
//#include "noise_generator.h"
#include "func_definitions.h"
#include "noises.h"

typedef enum DensityFnType {
    Add = 0,
    Mul = 1,
    Min = 2,
    Max = 3,
    Abs = 4,
    Square = 5,
    Cube = 6,
    HalfNegative = 7,
    QuarterNegative = 8,
    Squeeze = 9,
    Clamp = 10,
    YClampedGradient = 11,
    RangeChoice = 12,
    Noise = 13,
    ShiftedNoise = 14,
    Spline = 15,
    WeirdScaledSampler = 16,
    Interpolated = 17,
    BlendDensity = 18,
    BlendOffset = 19,
    BlendAlpha = 20,
    CacheOnce = 21,
    FlatCache = 22,
    Cache2D = 23,
    ShiftA = 24,
    ShiftB = 25,
    OldBlendedNoise = 26,
    EndIslands = 27,
} DensityFnType;

typedef enum DensityArgType {
    Constant = 0,
    Function = 1,
} DensityArgType;

typedef struct FunctionStackFrame {
    // The DensityFnType of this stack frame
    uchar fn_type;
    // The register position to store the result of this function
    uchar reg_position;
    // Arg type of 0 (Constant) contained in global constant_args
    // Arg type of 1 (Function) contained in private result_register
    uchar arg_types[3];
    // The position in constant_args or result_register where this function's args are stored
    ushort arg_positions[3];
    ushort constants_index;
    uchar noise_index;
} FunctionStackFrame;

typedef struct DensityFnState {
    int x;
    int y;
    int z;
    //__global const int* function_flow;
    //int flow_offset;
    //__global const int* flow_arg_types;
    //int flow_args_offset;
    __global const double* constant_args;
    //int constant_args_offset;
    __constant const VanillaNoise* noise_states;
    //int noise_states_offset;
    __constant const ImprovedNoise* noise_levels;
    //int noise_levels_offset;
    // noise amplitude offset is based on above noise_levels_offset
    __constant const double* noise_amplitudes;
    //int c_count;
    double arg_register[80];
    __constant const FunctionStackFrame* stack;
    short stack_offset;
    char skip_result;
} DensityFnState;

// Compute density function
inline void compute_function(DensityFnType f, DensityFnState* state) {
    //printf("cl function %d, %d\n", state->c_count, f);
    //printf("const pos %d\n", state->constant_args_offset_tmp);
    double result;
    switch (f) {
        case Add:
            result = add_compute(state);
            break;
        case Mul:
            result = mul_compute(state);
            break;
        case Min:
            result = min_compute(state);
            break;
        case Max:
            result = max_compute(state);
            break;
        case Abs:
            result = abs_compute(state);
            break;
        case Square:
            result = square_compute(state);
            break;
        case Cube:
            result = cube_compute(state);
            break;
        case HalfNegative:
            result = half_negative_compute(state);
            break;
        case QuarterNegative:
            result = quarter_negative_compute(state);
            break;
        case Squeeze:
            result = squeeze_compute(state);
            break;
        case Clamp:
            result = clamp_compute(state);
            break;
        case YClampedGradient:
            result = y_clamped_gradient_compute(state);
            break;
        case RangeChoice:
            result = range_choice_compute(state);
            break;
        case Noise:
            result = noise_compute(state);
            break;
        case ShiftedNoise:
            result = shifted_noise_compute(state);
            break;
        case Spline:
            result = spline_compute(state);
            break;
        case WeirdScaledSampler:
            result = weird_scaled_sampler_compute(state);
            break;
        case Interpolated:
            result = interpolated_compute(state);
            break;
        case BlendDensity:
            result = blend_density_compute(state);
            break;
        case BlendOffset:
            result = 0.0; // blend offset
            break;
        case BlendAlpha:
            result = 1.0; // blend alpha
            break;
        case CacheOnce:
            result = cache_once_compute(state);
            break;
        case FlatCache:
            result = flat_cache_compute(state);
            break;
        case Cache2D:
            result = cache_2d_compute(state);
            break;
        case ShiftA:
            result = shift_a_compute(state);
            break;
        case ShiftB:
            result = shift_b_compute(state);
            break;
        case OldBlendedNoise:
            result = old_blended_noise_compute(state);
            break;
        case EndIslands:
            result = 1.0;
            break;
        default:
            result = 0.0;
            break;
    }
    if (state->skip_result == 0) {
        state->arg_register[state->stack[state->stack_offset].reg_position] = result;
    } else {
        state->skip_result = 0;
    }
}

inline double get_stack_arg(DensityFnState* state, uchar arg_num) {
    ushort pos = state->stack[state->stack_offset].arg_positions[arg_num];
    return state->stack[state->stack_offset].arg_types[arg_num] == Constant
        ? state->constant_args[pos]
        : state->arg_register[pos];
}

inline double get_constant_arg(DensityFnState* state, uchar arg_num) {
    ushort pos = state->stack[state->stack_offset].constants_index;
    return state->constant_args[pos + arg_num];
}

// change to uchar
inline int get_function_const_skip(DensityFnType f) {
    switch (f) {
        case Clamp: // clamp (min, max)
            return 2;
        case YClampedGradient:
            return 4;
        case RangeChoice: // range choice (min, max)
            return 2;
        case Noise: // noise (xz, y scale)
            return 2;
        case ShiftedNoise: // shifted noise (xz, y scale)
            return 2;
        case Spline: // spline (point count and nested counts)
            return 6;
        case WeirdScaledSampler: // weird sampler (map type)
            return 1;
        case OldBlendedNoise:
            return 5;
        default:
            return 0; 
    }
}

// change to uchar
inline int get_function_arg_skip(DensityFnType f) {
    switch (f) {
        case 0:
            return 2;
        case 1:
            return 2;
        case 2:
            return 2;
        case 3:
            return 2;
        case 4:
            return 1;
        case 5:
            return 1;
        case 6:
            return 1;
        case 7:
            return 1;
        case 8:
            return 1;
        case 9:
            return 1;
        case 10:
            return 1;
        case 11:
            return 0;
        case 12:
            return 3;
        case 13:
            return 0;
        case 14:
            return 3;
        case 15:
            return 1;
        case 16:
            return 1;
        case 17:
            return 1;
        case 18:
            return 1;
        case 21:
            return 1;
        case 22:
            return 1;
        case 23:
            return 1;
        case 24:
            return 0;
        case 25:
            return 0;
        case 26:
            return 0;
    }
}

// change to uchar
inline int get_function_noise_skip(DensityFnType f) {
    switch (f) {
        case 13:
            return 1;
        case 14:
            return 1;
        case 16:
            return 1;
        case 24:
            return 1;
        case 25:
            return 1;
        default:
            return 0;
    }
}

#endif
