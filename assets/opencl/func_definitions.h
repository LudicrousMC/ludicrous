#ifndef FUNC_DEFINITIONS_H
#define FUNC_DEFINITIONS_H
struct DensityFnState;

double add_compute(struct DensityFnState* state);
double mul_compute(struct DensityFnState* state);
double min_compute(struct DensityFnState* state);
double max_compute(struct DensityFnState* state);
double abs_compute(struct DensityFnState* state);
double square_compute(struct DensityFnState* state);
double cube_compute(struct DensityFnState* state);
double half_negative_compute(struct DensityFnState* state);
double quarter_negative_compute(struct DensityFnState* state);
double squeeze_compute(struct DensityFnState* state);
double clamp_compute(struct DensityFnState* state);
double y_clamped_gradient_compute(struct DensityFnState* state);
double range_choice_compute(struct DensityFnState* state);
double noise_compute(struct DensityFnState* state);
double shifted_noise_compute(struct DensityFnState* state);
double spline_compute(struct DensityFnState* state);
double weird_scaled_sampler_compute(struct DensityFnState* state);
double interpolated_compute(struct DensityFnState* state);
double blend_density_compute(struct DensityFnState* state);
double cache_once_compute(struct DensityFnState* state);
double flat_cache_compute(struct DensityFnState* state);
double cache_2d_compute(struct DensityFnState* state);
double shift_a_compute(struct DensityFnState* state);
double shift_b_compute(struct DensityFnState* state);
double old_blended_noise_compute(struct DensityFnState* state);

typedef struct {
    double derivative;
    double location;
    double value;
} SplinePoint;

#endif
