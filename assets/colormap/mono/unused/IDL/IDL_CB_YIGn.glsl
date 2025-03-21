// Source from: https://github.com/kbinani/colormap-shaders
float colormap_red(float x) {
	if (x < 0.6289940178394318) {
		return ((5.11696133375750E+02 * x - 8.41303218634799E+02) * x + 2.59644604131609E+01) * x + 2.53419788769069E+02;
	} else {
		return (-1.59266359984140E+02 * x - 2.05336652594121E+01) * x + 1.40166356828632E+02;
	}
}

float colormap_green(float x) {
	if (x < 0.7657577693462372) {
		return ((1.07839212106774E+00 * x - 1.91222692546201E+02) * x - 1.99072424271726E+01) * x + 2.55781815734834E+02;
	} else {
		return (-2.23913463560442E+02 * x + 1.41364491447333E+02) * x + 1.51940340657207E+02;
	}
}

float colormap_blue(float x) {
	if (x < 0.1102530658245087) {
		return (-1.06015269490890E+01 * x - 3.53323657599336E+02) * x + 2.28818914956011E+02;
	} else if (x < 0.7528357207775116) {
		return (((6.02578715483658E+02 * x - 1.15715802822919E+03) * x + 7.15916939803876E+02) * x - 3.43088208134950E+02) * x + 2.19476164232866E+02;
	} else {
		return (-6.54502605847083E+01 * x + 1.22002989630042E+01) * x + 9.46764726107154E+01;
	}
}

vec3 IDL_CB_YIGn_colormap(float x) {
	float r = clamp(colormap_red(x) / 255.0, 0.0, 1.0);
	float g = clamp(colormap_green(x) / 255.0, 0.0, 1.0);
	float b = clamp(colormap_blue(x) / 255.0, 0.0, 1.0);
	return vec3(r, g, b);
}
