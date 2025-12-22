#ifndef UTIL_H
#define UTIL_H

double lerp(double pos, double x, double y) {
    return x + (pos * (y - x));
}

double lerp2(
    double pos1,
    double pos2,
    double x1,
    double y1,
    double x2,
    double y2
) {
    return lerp(pos2, lerp(pos1, x1, y1), lerp(pos1, x2, y2));
}

double lerp3(
    double pos1,
    double pos2,
    double pos3,
    double x1,
    double y1,
    double x2,
    double y2,
    double x3,
    double y3,
    double x4,
    double y4
) {
    return lerp(
        pos3,
        lerp2(pos1, pos2, x1, y1, x2, y2),
        lerp2(pos1, pos2, x3, y3, x4, y4)
    );
}

double inverse_lerp(double pos, double x, double y) {
    return (pos - x) / (y - x);
}

double smoothstep(double value) {
    return value * value * value * (value * (value * 6.0 - 15.0) + 10.0);
}

double clamped_map(
    double pos1,
    double x,
    double y,
    double val1,
    double val2
) {
    double inv_lerp = inverse_lerp(pos1, x, y);
    return inv_lerp < 0.0 ? val1 : inv_lerp > 1.0 ? val2 : lerp(inv_lerp, val1, val2);
}

#endif
