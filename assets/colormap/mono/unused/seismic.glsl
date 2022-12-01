// Note that this is NOT exactly same as matplotlib's seismic
// Especailly in blue region's end value, which is lower than this
// add by dgkim
vec3 seismic_colormap(float t) {
    vec3 low_color = vec3(0.0, 0.0, 1.0);
    vec3 high_color = vec3(1.0, 0.0, 0.0);
    vec3 mid_color = vec3(1.0, 1.0, 1.0);

    float interp = 1 - abs(1.0 - 2.0 * t); // 0 ~ 1 ~ 0
    float where_base = 0.5; // 0.0 leads to bwr color map

    vec3 base_color = t < 0.5 ? low_color : high_color;
    // Dark color
    vec3 base_color_dark = base_color * (1 - where_base);
    // base color in range of interp: 0 ~ 1, exceeding 1.0 will be clamped
    vec3 base_color_interp = base_color * interp + base_color_dark;
    // after passing where_base, negative value will be clamped
    vec3 other_color_interp = (mid_color - base_color) * (interp - where_base) / (1 - where_base);
    
    vec3 rgb = base_color_interp + other_color_interp;
    return vec3(clamp(rgb, 0.0, 1.0));
}
