vec3 copper_colormap (float x) {
  return vec3(
    clamp(80.0 / 63.0 * x + 5.0 / 252.0, 0.0, 1.0),
    clamp(0.7936 * x - 0.0124, 0.0, 1.0),
    clamp(796.0 / 1575.0 * x + 199.0 / 25200.0, 0.0, 1.0)
  );
}
