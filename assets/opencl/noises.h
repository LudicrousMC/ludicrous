#ifndef NOISES_H
#define NOISES_H

typedef struct {
    double x;
    double y;
    double z;
    int values[256];
    uchar disabled;
} ImprovedNoise;

typedef struct {
    uchar noise_count;
    ushort data_position;
    double lowest_val_factor;
    double lowest_input_factor;
} PerlinNoise;

typedef struct {
    PerlinNoise noises[2];
    double val_factor;
    double val_max;
} VanillaNoise;

#endif
