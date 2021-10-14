vec3 summer_colormap (float x) {
  return vec3(clamp(x, 0.0, 1.0), clamp(0.5 * x + 0.5, 0.0, 1.0), 0.4);
}
