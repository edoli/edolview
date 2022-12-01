vec3 cool_colormap (float x) {
  return vec3(
    clamp((1.0 + 1.0 / 63.0) * x - 1.0 / 63.0, 0.0, 1.0),
    clamp(-(1.0 + 1.0 / 63.0) * x + (1.0 + 1.0 / 63.0), 0.0, 1.0),
    1.0
  );
}