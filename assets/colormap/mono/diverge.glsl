vec3 lab2msh(vec3 lab) {
    float L = lab.r;
    float a = lab.g;
    float b = lab.b;
    float M = length(lab);

    return vec3(
        M, acos(L / M), atan(b, a)
    );
}

vec3 msh2lab(vec3 msh) {
    float M = msh.r;
    float s = msh.g;
    float h = msh.b;

    return vec3(
        M * cos(s),
        M * sin(s) * cos(h),
        M * sin(s) * sin(h)
    );
}

vec3 rgb2msh(vec3 rgb) {
    return lab2msh(xyz2lab(rgb2xyz(rgb)));
}

vec3 msh2rgb(vec3 msh) {
    return xyz2rgb(lab2xyz(msh2lab(msh)));
}

vec3 diverge_colormap(float t) {
    vec3 low_color = vec3(0.230, 0.299, 0.754);
    vec3 high_color = vec3(0.706, 0.016, 0.150);
    vec3 low_msh = vec3(80.0154, 1.0797, -1.1002);
    vec3 high_msh = vec3(80.0316, 1.0798, 0.5008);
    vec3 mid_msh = vec3(88.0, 0.0, 0.0);
    // vec3 mid_msh = rgb2msh(vec3(0.8654, 0.8654, 0.8654));

    float interp = abs(1.0 - 2.0 * t);

    // vec3 msh = rgb2msh(t < 0.5 ? low_color : high_color);
    vec3 msh = t < 0.5 ? low_msh : high_msh;
    msh.x = interp * msh.x + (1.0 - interp) * mid_msh.x;
    msh.y = interp * msh.y + (1.0 - interp) * mid_msh.y;
    vec3 rgb = msh2rgb(msh);
    return vec3(clamp(rgb, 0.0, 1.0));
}