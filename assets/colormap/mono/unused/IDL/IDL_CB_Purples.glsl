// Source from: https://github.com/kbinani/colormap-shaders
float colormap_red(float x) {
	return (((-3.03160346735036E+02 * x + 7.95537661771755E+02) * x - 6.68077287777559E+02) * x - 9.83523581613554E+00) * x + 2.49241870138829E+02;
}

float colormap_green(float x) {
	if (x < 0.6238275468349457) {
		return ((((-1.64962450015544E+03 * x + 3.91401450219750E+03) * x - 2.81559997409582E+03) * x + 5.71903768479824E+02) * x - 1.63510103061329E+02) * x + 2.52440721674553E+02;
	} else {
		return (8.00008172182588E+01 * x - 4.62535128626795E+02) * x + 3.83781070034667E+02;
	}
}

float colormap_blue(float x) {
	return (((((1.42855146044492E+03 * x - 4.10835541903972E+03) * x + 4.43536620247364E+03) * x - 2.15825854188203E+03) * x + 3.66481133684515E+02) * x - 9.02285603303462E+01) * x + 2.53802694290353E+02;
}

vec3 IDL_CB_Purples_colormap(float x) {
	float r = clamp(colormap_red(x) / 255.0, 0.0, 1.0);
	float g = clamp(colormap_green(x) / 255.0, 0.0, 1.0);
	float b = clamp(colormap_blue(x) / 255.0, 0.0, 1.0);
	return vec3(r, g, b);
}
