#version 120
#ifdef GL_ES
    #define LOWP lowp
    precision mediump float;
#else
    #define LOWP
#endif

#define PI 3.1415926535897932384626433832795

varying LOWP vec4 v_color;
varying vec2 v_texCoords;
uniform sampler2D u_texture;

uniform int width;
uniform int height;
uniform float offset;
uniform float exposure;
uniform float gamma;
uniform float minV;
uniform float maxV;
uniform int is_inverse;
uniform float r_scale;
uniform float g_scale;
uniform float b_scale;

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
    float t = pow(c, 3.0);
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

vec3 rgb2xyz(vec3 rgb) {
    return inv_gamma_f(rgb) * rgb2xyz_mat;
}

vec3 xyz2rgb(vec3 xyz) {
    return gamma_f(xyz * xyz2rgb_mat);
}

vec3 lab2rgb(vec3 lab) {
    return xyz2rgb(lab2xyz(lab));
}

vec3 rgb2lab(vec3 rgb) {
    return xyz2lab(rgb2xyz(rgb));
}

float color_proc(float v)
{
    float tmp = v * pow(2.0, exposure) + offset;
    return sign(tmp) * pow(abs(tmp), 1.0 / gamma);
}

%colormap_function%

void main()
{
    vec4 tex = texture2D(u_texture, v_texCoords);
    float p;
    float alpha = tex.a;

    tex.r *= r_scale;
    tex.g *= g_scale;
    tex.b *= b_scale;

    float image_x = v_texCoords.x * float(width);
    float image_y = v_texCoords.y * float(height);

    if (is_inverse == 1) {
        float invMinV = 1.0 / minV;
        float invMaxV = 1.0 / maxV;
        tex = ((1.0 / tex) - invMaxV) / (invMinV - invMaxV);
    } else {
        tex = (tex - minV) / (maxV - minV);
    }

    %color_process%

    %final_shader%
}
