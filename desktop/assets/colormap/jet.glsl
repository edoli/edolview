float jet_colormap_red(float x) {
    if (x < 0.7) {
        return 4.0 * x - 1.5;
    } else {
        return -4.0 * x + 4.5;
    }
}

float jet_colormap_green(float x) {
    if (x < 0.5) {
        return 4.0 * x - 0.5;
    } else {
        return -4.0 * x + 3.5;
    }
}

float jet_colormap_blue(float x) {
    if (x < 0.3) {
        return 4.0 * x + 0.5;
    } else {
        return -4.0 * x + 2.5;
    }
}

vec3 jet_colormap(float x) {
    float r = clamp(jet_colormap_red(x), 0.0, 1.0);
    float g = clamp(jet_colormap_green(x), 0.0, 1.0);
    float b = clamp(jet_colormap_blue(x), 0.0, 1.0);
    return vec3(r, g, b);
}