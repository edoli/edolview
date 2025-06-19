// Red-Black-Green colormap
// Note that the end color is not the exact green (0, 1, 0) and red (1, 0, 0)

vec3 rkg_colormap(float t) {
    vec3 low_color = vec3(1.0, 0.15, 0.15);
    vec3 high_color = vec3(0.15, 1.0, 0.15);

    float interp = abs(1.0 - 2.0 * t); // 0 ~ 1 ~ 0

    vec3 end_color = t < 0.5 ? low_color : high_color;
    vec3 rgb = end_color * interp;
    return clamp(rgb, 0.0, 1.0);
}
