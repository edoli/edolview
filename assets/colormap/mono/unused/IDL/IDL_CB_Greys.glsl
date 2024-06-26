// Source from: https://github.com/kbinani/colormap-shaders
float colormap_f1(float x) {
	if (x < 0.3849871446504941) {
		return (-1.97035589869658E+02 * x - 1.04694505989261E+02) * x + 2.54887830314633E+02;
	} else if (x < 0.7524552013985151) {
		return (8.71964614639801E+01 * x - 3.79941007690502E+02) * x + 3.18726712728548E+02;
	} else {
		return (2.28085532626505E+02 * x - 7.25770100421835E+02) * x + 4.99177793972139E+02;
	}
}

vec3 IDL_CB_Greys_colormap(float x) {
	float v = clamp(colormap_f1(x) / 255.0, 0.0, 1.0);
	return vec3(v, v, v);
}
