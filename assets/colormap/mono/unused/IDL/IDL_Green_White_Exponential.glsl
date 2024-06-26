// Source from: https://github.com/kbinani/colormap-shaders
float colormap_red(float x) {
    if (x < (5.33164074896858E-01 + 3.69558823529412E+01) / (6.61764705882353E-01 - 3.80845483226613E-01)) { // 133.451339048
        return 3.80845483226613E-01 * x + 5.33164074896858E-01;
    } else if(x < (2.21853643274093E+02 - 3.69558823529412E+01) / (1.86816585713397E+00 - 6.61764705882353E-01)) { // 153.263912861
        return 6.61764705882353E-01 * x - 3.69558823529412E+01;
    } else {
        return 1.86816585713397E+00 * x - 2.21853643274093E+02;
    }
}

float colormap_green(float x) {
    if (x < (8.74223522059742E+01 - 3.33294186729301E-01) / (1.34076340457443E+00 - 6.66705813270699E-01)) { // 129.201212393
        return 6.66705813270699E-01 * x - 3.33294186729301E-01;
    } else {
        return 1.34076340457443E+00 * x - 8.74223522059742E+01;
    }
}

float colormap_blue(float x) {
    if (x < (4.92898927047827E+02 - 4.63219741480611E-01) / (2.93126567624928E+00 - 2.63081042553601E-01)) {
        return 2.63081042553601E-01 * x - 4.63219741480611E-01;
    } else {
        return 2.93126567624928E+00 * x - 4.92898927047827E+02;
    }
}

vec3 IDL_Green_White_Exponential_colormap(float x) {
    float t = x * 255.0;
    float r = clamp(colormap_red(t) / 255.0, 0.0, 1.0);
    float g = clamp(colormap_green(t) / 255.0, 0.0, 1.0);
    float b = clamp(colormap_blue(t) / 255.0, 0.0, 1.0);
	return vec3(r, g, b);
}
