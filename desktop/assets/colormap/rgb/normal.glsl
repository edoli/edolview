vec3 normal_colormap(vec3 t) {
    vec3 nt = normalize(t);

    return vec3((nt.r + 1.0) * 0.5, (nt.g + 1.0) * 0.5, nt.b + 0.5);
}