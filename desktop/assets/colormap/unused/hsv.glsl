float hsv_colormap_red(float x) {
    if (x < 0.5) {
        return -6.0 * x + 67.0 / 32.0;
    } else {
        return 6.0 * x - 79.0 / 16.0;
    }
}

float hsv_colormap_green(float x) {
    if (x < 0.4) {
        return 6.0 * x - 3.0 / 32.0;
    } else {
        return -6.0 * x + 79.0 / 16.0;
    }
}

float hsv_colormap_blue(float x) {
    if (x < 0.7) {
        return 6.0 * x - 67.0 / 32.0;
    } else {
        return -6.0 * x + 195.0 / 32.0;
    }
}

vec3 hsv_colormap(float x) {
    float r = clamp(hsv_colormap_red(x), 0.0, 1.0);
    float g = clamp(hsv_colormap_green(x), 0.0, 1.0);
    float b = clamp(hsv_colormap_blue(x), 0.0, 1.0);
    return vec3(r, g, b);
}