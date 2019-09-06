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

void main()
{
    vec4 p = texture2D(u_texture, v_texCoords);
    if (normalize == 1) {
        p = (p - min) / (max - min);
    }
    gl_FragColor = v_color * ((pow(p, vec4(1.0 / gamma)) + brightness - 0.5) * contrast + 0.5);
}