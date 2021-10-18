vec3 spring_colormap (float x) {
  return vec3(1.0, clamp(x, 0.0, 1.0), clamp(1.0 - x, 0.0, 1.0));
}
