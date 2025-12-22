#ifndef NOISE_GENERATOR_H
#define NOISE_GENERATOR_H
#include "util.h"
#include "func_state.h"

__constant char SIMPLEX_GRADIENT[16][3] = {
    { 1,  1,  0},
    {-1,  1,  0},
    { 1, -1,  0},
    {-1, -1,  0},
    { 1,  0,  1},
    {-1,  0,  1},
    { 1,  0, -1},
    {-1,  0, -1},
    { 0,  1,  1},
    { 0, -1,  1},
    { 0,  1, -1},
    { 0, -1, -1},
    { 1,  1,  0},
    { 0, -1,  1},
    {-1,  1,  0},
    { 0, -1, -1}
};

inline double sample_plus_lerp(
    __constant const ImprovedNoise* noise,
    int x_floor,
    int y_floor,
    int z_floor,
    double x,
    double y,
    double z,
    double y_offset
) {
    int val1 = noise->values[x_floor & 0xFF] & 0xFF;
    int val2 = noise->values[(x_floor + 1) & 0xFF] & 0xFF;
    int val3 = noise->values[(y_floor + val1) & 0xFF] & 0xFF;
    int val4 = noise->values[(y_floor + val1 + 1) & 0xFF] & 0xFF;
    int val5 = noise->values[(y_floor + val2) & 0xFF] & 0xFF;
    int val6 = noise->values[(y_floor + val2 + 1) & 0xFF] & 0xFF;

    int val7 = noise->values[(z_floor + val3) & 0xFF] & 0xF;
    int val8 = noise->values[(z_floor + val5) & 0xFF] & 0xF;
    int val9 = noise->values[(z_floor + val4) & 0xFF] & 0xF;
    int val10 = noise->values[(z_floor + val6) & 0xFF] & 0xF;
    int val11 = noise->values[(z_floor + val3 + 1) & 0xFF] & 0xF;
    int val12 = noise->values[(z_floor + val5 + 1) & 0xFF] & 0xF;
    int val13 = noise->values[(z_floor + val4 + 1) & 0xFF] & 0xF;
    int val14 = noise->values[(z_floor + val6 + 1) & 0xFF] & 0xF;
    
    double x1 = (double)SIMPLEX_GRADIENT[val7][0] * x
        + (double)SIMPLEX_GRADIENT[val7][1] * y_offset
        + (double)SIMPLEX_GRADIENT[val7][2] * z;
    double y1 = (double)SIMPLEX_GRADIENT[val8][0] * (x - 1.0)
        + (double)SIMPLEX_GRADIENT[val8][1] * y_offset
        + (double)SIMPLEX_GRADIENT[val8][2] * z;
    double x2 = (double)SIMPLEX_GRADIENT[val9][0] * x
        + (double)SIMPLEX_GRADIENT[val9][1] * (y_offset - 1.0)
        + (double)SIMPLEX_GRADIENT[val9][2] * z;
    double y2 = (double)SIMPLEX_GRADIENT[val10][0] * (x - 1.0)
        + (double)SIMPLEX_GRADIENT[val10][1] * (y_offset - 1.0)
        + (double)SIMPLEX_GRADIENT[val10][2] * z;
    double x3 = (double)SIMPLEX_GRADIENT[val11][0] * x
        + (double)SIMPLEX_GRADIENT[val11][1] * y_offset
        + (double)SIMPLEX_GRADIENT[val11][2] * (z - 1.0);
    double y3 = (double)SIMPLEX_GRADIENT[val12][0] * (x - 1.0)
        + (double)SIMPLEX_GRADIENT[val12][1] * y_offset
        + (double)SIMPLEX_GRADIENT[val12][2] * (z - 1.0);
    double x4 = (double)SIMPLEX_GRADIENT[val13][0] * x
        + (double)SIMPLEX_GRADIENT[val13][1] * (y_offset - 1.0)
        + (double)SIMPLEX_GRADIENT[val13][2] * (z - 1.0);
    double y4 = (double)SIMPLEX_GRADIENT[val14][0] * (x - 1.0)
        + (double)SIMPLEX_GRADIENT[val14][1] * (y_offset - 1.0)
        + (double)SIMPLEX_GRADIENT[val14][2] * (z - 1.0);

    return lerp3(
        smoothstep(x),
        smoothstep(y),
        smoothstep(z),
        x1,
        y1,
        x2,
        y2,
        x3,
        y3,
        x4,
        y4
    );
}

inline double generate(
    __constant const ImprovedNoise* noise,
    double x,
    double y,
    double z,
    double val1, 
    double val2
) {
    x += noise->x;
    y += noise->y;
    z += noise->z;
    int x_floor = (int)floor(x);
    int y_floor = (int)floor(y);
    int z_floor = (int)floor(z);
    x -= x_floor;
    y -= y_floor;
    z -= z_floor;
    double y_offset;
    if (val1 != 0.0) {
        double val = val2 >= 0.0 && val2 < y ? val2 : y;
        y_offset = y - (val1 * floor((val / val1) + 1.0e-7));
    } else {
        y_offset = y;
    }
    return sample_plus_lerp(
        noise,
        x_floor,
        y_floor,
        z_floor,
        x, y, z, y_offset
    );
}

inline double wrap(double value) {
    return value - floor(value / (double)(1 << 25)) * (double)(1 << 25);
}

inline double get_perlin_val(__constant const PerlinNoise* noise, DensityFnState* state, double x, double y, double z) {
    double value = 0.0;
    double input_factor = noise->lowest_input_factor;
    double value_factor = noise->lowest_val_factor;
    ushort offset = noise->data_position;
    for(int i = 0; i < noise->noise_count; i++) {
        if (state->noise_levels[i + offset].disabled == 0) {
            value += state->noise_amplitudes[i + offset]
                * generate(
                    &state->noise_levels[i + offset],
                    wrap(x * input_factor),
                    wrap(y * input_factor),
                    wrap(z * input_factor),
                    0.0,
                    0.0
                )
                * value_factor;
        }
        input_factor *= 2.0;
        value_factor /= 2.0;
    }
    return value;
}

inline int get_perlin_noise_count(PerlinNoise* noise) {
    return (int)noise->noise_count;
}

inline __constant const ImprovedNoise* get_perlin_level(__constant const PerlinNoise* noise, DensityFnState* state, int level) {
    return &state->noise_levels[noise->noise_count - level - 1 + noise->data_position];
}

inline double get_vanilla_val(
    DensityFnState* state,
    double x,
    double y,
    double z
) {
    // Get noise state of current stack frame
    __constant const VanillaNoise* noise = &state->noise_states[state->stack[state->stack_offset].noise_index];
    return (
        get_perlin_val(
            &noise->noises[0],
            state,
            x, y, z
        ) + get_perlin_val(
            &noise->noises[1],
            state,
            x * 1.0181268882175227,
            y * 1.0181268882175227,
            z * 1.0181268882175227
        )
    ) * noise->val_factor;
}

/*inline VanillaNoiseState get_noise_state(struct DensityFnState* state);
inline VanillaNoiseState get_first_noise_state(struct DensityFnState* state);
inline int get_noise_offset(struct DensityFnState* state);

inline VanillaNoise build_vanilla_noise(struct DensityFnState* state) {
    VanillaNoiseState vanilla_noise_state = get_noise_state(state);
    int noise_offset = get_noise_offset(state);
    PerlinNoiseState noise1_state = vanilla_noise_state.noises[0];
    PerlinNoiseState noise2_state = vanilla_noise_state.noises[1];
    PerlinNoise noise1 = {
        noise1_state.noise_count,
        noise1_state.lowest_val_factor,
        noise1_state.lowest_input_factor,
        noise_offset,
    };
    PerlinNoise noise2 = {
        noise2_state.noise_count,
        noise2_state.lowest_val_factor,
        noise2_state.lowest_input_factor,
        noise_offset + noise1_state.noise_count,
    };
    VanillaNoise noise_generator = {
        noise1,
        noise2,
        vanilla_noise_state.val_factor,
        vanilla_noise_state.val_max
    };
    return noise_generator;
}

inline VanillaNoise build_old_blended_noise(struct DensityFnState* state) {
    VanillaNoiseState vanilla_noise_state = get_first_noise_state(state);
    PerlinNoiseState noise1_state = vanilla_noise_state.noises[0];
    PerlinNoiseState noise2_state = vanilla_noise_state.noises[1];
    PerlinNoise noise1 = {
        noise1_state.noise_count,
        noise1_state.lowest_val_factor,
        noise1_state.lowest_input_factor,
        0,
    };
    PerlinNoise noise2 = {
        noise2_state.noise_count,
        noise2_state.lowest_val_factor,
        noise2_state.lowest_input_factor,
        noise1_state.noise_count,
    };
    VanillaNoise noise_generator = {
        noise1,
        noise2,
        vanilla_noise_state.val_factor,
        vanilla_noise_state.val_max
    };
    return noise_generator;
}*/

#endif
