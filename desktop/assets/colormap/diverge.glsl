// https://www.kennethmoreland.com/color-maps/

mat3 rgb2xyz_mat = mat3(
    0.4124564, 0.3575761, 0.1804375,
    0.2126729, 0.7151522, 0.0721750,
    0.0193339, 0.1191920, 0.9503041
);

mat3 xyz2rgb_mat = mat3(
    3.2404542, -1.5371385, -0.4985314,
    -0.9692660, 1.8760108, 0.0415560,
    0.0556434, -0.2040259, 1.0572252
);

// d65 white reference
float xn = 0.95047;
float yn = 1.0;
float zn = 1.08883;

vec3 gamma_f(vec3 rgb) {
    return vec3(
        pow(rgb.r, 1.0 / 2.2),
        pow(rgb.g, 1.0 / 2.2),
        pow(rgb.b, 1.0 / 2.2)
    );
}

vec3 inv_gamma_f(vec3 rgb) {
    return vec3(
    pow(rgb.r, 2.2),
    pow(rgb.g, 2.2),
    pow(rgb.b, 2.2)
    );
}

vec3 rgb2xyz(vec3 rgb) {
    return inv_gamma_f(rgb) * rgb2xyz_mat;
}

vec3 xyz2rgb(vec3 xyz) {
    return gamma_f(xyz * xyz2rgb_mat);
}

float f(float c) {
    return c > 0.008856 ? pow(c, 1.0 / 3.0) : (903.3 * c + 16.0) / 116.0;
}

vec3 xyz2lab(vec3 xyz){
    float fx = f(xyz.x / xn);
    float fy = f(xyz.y / yn);
    float fz = f(xyz.z / zn);
    return vec3(
        116.0 * fx - 16.0,
        500.0 * (fx - fy),
        200.0 * (fy - fz)
    );
}

float f_inv(float c) {
    float t = pow(c, 3);
    return t > 0.008856 ? t : (116.0 * c - 16.0) / 903.3;
}


vec3 lab2xyz(vec3 lab){
    float L = lab.x;
    float a = lab.y;
    float b = lab.z;

    float fy = (L + 16.0) / 116.0;
    float fz = fy - b / 200.0;
    float fx = a / 500.0 + fy;

    return vec3(f_inv(fx) * xn, f_inv(fy) * yn, f_inv(fz) * zn);
}

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

vec3 lab2rgb(vec3 lab) {
    return xyz2rgb(lab2xyz(lab));
}

vec3 rgb2lab(vec3 rgb) {
    return xyz2lab(rgb2xyz(rgb));
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