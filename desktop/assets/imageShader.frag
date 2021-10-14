#ifdef GL_ES
#define LOWP lowp
precision mediump float;
#else
#define LOWP
#endif
varying LOWP vec4 v_color;
varying vec2 v_texCoords;
uniform sampler2D u_texture;

uniform int width;
uniform int height;
uniform float brightness;
uniform float contrast;
uniform float gamma;
uniform float minV;
uniform float maxV;
uniform int normalize;

%colormap_function%

void main()
{
    vec4 tex = texture2D(u_texture, v_texCoords);
    float p;
    float alpha = tex.a;

    float image_x = v_texCoords.x * float(width);
    float image_y = v_texCoords.y * float(height);

    tex = (tex - minV) / (maxV - minV);

    %extra_code%
    %color_process%
}
