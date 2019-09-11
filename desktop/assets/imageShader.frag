#ifdef GL_ES
#define LOWP lowp
precision mediump float;
#else
#define LOWP
#endif
varying LOWP vec4 v_color;
varying vec2 v_texCoords;
uniform sampler2D u_texture;

uniform float brightness;
uniform float contrast;
uniform float gamma;
uniform float min;
uniform float max;
uniform int normalize;
uniform int colormap;


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

vec4 jet_colormap(float x) {
    float r = clamp(jet_colormap_red(x), 0.0, 1.0);
    float g = clamp(jet_colormap_green(x), 0.0, 1.0);
    float b = clamp(jet_colormap_blue(x), 0.0, 1.0);
    return vec4(r, g, b, 1.0);
}

void main()
{
    vec4 p = texture2D(u_texture, v_texCoords);
    if (normalize == 1) {
        p = (p - min) / (max - min);
    }
    if (colormap == 0) {
        gl_FragColor = v_color * ((pow(p, vec4(1.0 / gamma)) + brightness - 0.5) * contrast + 0.5);
    } else {
        gl_FragColor = jet_colormap((pow(p.r, (1.0 / gamma)) + brightness - 0.5) * contrast + 0.5);
    }
}
