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

vec3 jet_colormap(float x) {
    float r = clamp(jet_colormap_red(x), 0.0, 1.0);
    float g = clamp(jet_colormap_green(x), 0.0, 1.0);
    float b = clamp(jet_colormap_blue(x), 0.0, 1.0);
    return vec3(r, g, b);
}

vec3 hot_colormap(float x) {
    float r = clamp(8.0 / 3.0 * x, 0.0, 1.0);
    float g = clamp(8.0 / 3.0 * x - 1.0, 0.0, 1.0);
    float b = clamp(4.0 * x - 3.0, 0.0, 1.0);
    return vec3(r, g, b);
}

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

vec3 viridis_colormap(float t) {
    const vec3 c0 = vec3(0.2777273272234177, 0.005407344544966578, 0.3340998053353061);
    const vec3 c1 = vec3(0.1050930431085774, 1.404613529898575, 1.384590162594685);
    const vec3 c2 = vec3(-0.3308618287255563, 0.214847559468213, 0.09509516302823659);
    const vec3 c3 = vec3(-4.634230498983486, -5.799100973351585, -19.33244095627987);
    const vec3 c4 = vec3(6.228269936347081, 14.17993336680509, 56.69055260068105);
    const vec3 c5 = vec3(4.776384997670288, -13.74514537774601, -65.35303263337234);
    const vec3 c6 = vec3(-5.435455855934631, 4.645852612178535, 26.3124352495832);
    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));
}

vec3 plasma_colormap(float t) {
    const vec3 c0 = vec3(0.05873234392399702, 0.02333670892565664, 0.5433401826748754);
    const vec3 c1 = vec3(2.176514634195958, 0.2383834171260182, 0.7539604599784036);
    const vec3 c2 = vec3(-2.689460476458034, -7.455851135738909, 3.110799939717086);
    const vec3 c3 = vec3(6.130348345893603, 42.3461881477227, -28.51885465332158);
    const vec3 c4 = vec3(-11.10743619062271, -82.66631109428045, 60.13984767418263);
    const vec3 c5 = vec3(10.02306557647065, 71.41361770095349, -54.07218655560067);
    const vec3 c6 = vec3(-3.658713842777788, -22.93153465461149, 18.19190778539828);
    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));
}

vec3 magma_colormap(float t) {
    const vec3 c0 = vec3(-0.002136485053939582, -0.000749655052795221, -0.005386127855323933);
    const vec3 c1 = vec3(0.2516605407371642, 0.6775232436837668, 2.494026599312351);
    const vec3 c2 = vec3(8.353717279216625, -3.577719514958484, 0.3144679030132573);
    const vec3 c3 = vec3(-27.66873308576866, 14.26473078096533, -13.64921318813922);
    const vec3 c4 = vec3(52.17613981234068, -27.94360607168351, 12.94416944238394);
    const vec3 c5 = vec3(-50.76852536473588, 29.04658282127291, 4.23415299384598);
    const vec3 c6 = vec3(18.65570506591883, -11.48977351997711, -5.601961508734096);
    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));
}

vec3 inferno_colormap(float t) {
    const vec3 c0 = vec3(0.0002189403691192265, 0.001651004631001012, -0.01948089843709184);
    const vec3 c1 = vec3(0.1065134194856116, 0.5639564367884091, 3.932712388889277);
    const vec3 c2 = vec3(11.60249308247187, -3.972853965665698, -15.9423941062914);
    const vec3 c3 = vec3(-41.70399613139459, 17.43639888205313, 44.35414519872813);
    const vec3 c4 = vec3(77.162935699427, -33.40235894210092, -81.80730925738993);
    const vec3 c5 = vec3(-71.31942824499214, 32.62606426397723, 73.20951985803202);
    const vec3 c6 = vec3(25.13112622477341, -12.24266895238567, -23.07032500287172);
    return c0+t*(c1+t*(c2+t*(c3+t*(c4+t*(c5+t*c6)))));
}

void main()
{
    vec4 tex = texture2D(u_texture, v_texCoords);
    float p;
    float alpha = tex.a;
    tex = (tex - min) / (max - min);

    if (colormap == 0) {
        // pow(p * contrast + brightness, 1.0 / gamma)
        vec4 v;
        p = tex.r;
        v.r = %pixel_expression%;
        p = tex.g;
        v.g = %pixel_expression%;
        p = tex.b;
        v.b = %pixel_expression%;
        gl_FragColor = v_color * v;
        gl_FragColor.a = alpha;
    } else {
        // pow(p * contrast + brightness, 1.0 / gamma)
        p = tex.r;
        float v = %pixel_expression%;
        v = clamp(v, 0.0, 1.0);
        vec3 color;
        if (colormap == 1) {
            color = jet_colormap(v);
        } else if (colormap == 2) {
            color = hot_colormap(v);
        } else if (colormap == 3) {
            color = hsv_colormap(v);
        } else if (colormap == 4) {
            color = viridis_colormap(v);
        } else if (colormap == 5) {
            color = plasma_colormap(v);
        } else if (colormap == 6) {
            color = magma_colormap(v);
        } else if (colormap == 7) {
            color = inferno_colormap(v);
        }
        gl_FragColor = vec4(color.r, color.g, color.b, alpha);
    }
}
