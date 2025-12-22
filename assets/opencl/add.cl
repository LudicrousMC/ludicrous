#pragma OPENCL EXTENSION cl_khr_fp64 : enable
#include "noise_generator.h"

typedef struct {
    int x;
    int y;
    int z;
} BlockPos;

BlockPos unpack_xyz(ulong packed) {
    BlockPos pos = {
        (int)((packed >> 38) & 0x3FFFFFFUL) - (1 << 25),
        (int)((packed >> 26) & 0xFFFUL) - (1 << 11),
        (int)(packed & 0x3FFFFFFUL) - (1 << 25)
    };
    return pos;
}

__kernel void add(
    //__global const long* positions,
    __global const VanillaNoise* vanilla_noise,
    __global const ImprovedNoise* noise_levels,
    __global const double* noise_amplitudes,
    __global double* result
) {
    int i = get_global_id(0);
    /*PerlinNoiseState noise1_state = vanilla_noise->noises[0];
    PerlinNoiseState noise2_state = vanilla_noise->noises[1];
PerlinNoise noise1 = {
        noise1_state.noise_count,
        noise1_state.lowest_val_factor,
        noise1_state.lowest_input_factor,
        0,
    };
    int noise2_buf_offset = noise2_state.noise_count;
    PerlinNoise noise2 = {
        noise2_state.noise_count,
        noise2_state.lowest_val_factor,
        noise2_state.lowest_input_factor,
        noise1_state.noise_count,
    };
    VanillaNoise noise_generator = {
        noise1,
        noise2,
        vanilla_noise->val_factor
    };*/

    //BlockPos pos = unpack_xyz(positions[i]);
    //result[i] = get_vanilla_val(&noise_generator, -10000000.0, 10.0, -20.0);
/*int x = get_global_id(0);
    int y = get_global_id(1);
    int z = get_global_id(2);
    int block = (double)x + (double)y * 16 + (double)z * 256;*/
}
