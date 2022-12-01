#ifdef GL_ES
#define LOWP lowp
precision mediump float;
#else
#define LOWP
#endif

uniform float u_grid_size;
uniform vec4 grid_color_a;
uniform vec4 grid_color_b;

varying vec2 v_position;

void main()
{
    float image_x = v_position.x;
    float image_y = v_position.y;
    float grid_size_double = u_grid_size * 2.0;
    gl_FragColor = (mod(image_x, grid_size_double) < u_grid_size) != (mod(image_y, grid_size_double) < u_grid_size) ? grid_color_a : grid_color_b;
}
