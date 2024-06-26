// Source from: https://github.com/kbinani/colormap-shaders
float colormap_red(float x) {
	if (x < 0.4668049514293671) {
		return -1.36007661451525E+01 * x + 2.54876081825334E+02;
	} else {
		return ((9.11043267377652E+02 * x - 2.27422817830303E+03) * x + 1.47691217772832E+03) * x - 3.80041369120933E+01;
	}
}

float colormap_green(float x) {
	return ((((-2.12978937384858E+03 * x + 5.05211767883971E+03) * x - 3.95843947196006E+03) * x + 9.49632208843715E+02) * x - 2.70366761763812E+02) * x + 2.48595803511253E+02;
}

float colormap_blue(float x) {
	if (x < 0.2484821379184723) {
		return ((1.12923860577866E+02 * x - 2.02431339810602E+02) * x - 1.60306874714734E+02) * x + 2.42581612831587E+02;
	} else if (x < 0.5019654333591461) {
		return (-2.24073120483401E+02 * x + 4.46032892337713E+01) * x + 1.94733826112356E+02;
	} else if (x < 0.7505462467670441) {
		return (-4.08932859712077E+02 * x + 3.70448937862306E+02) * x + 7.77495522761299E+01;
	} else {
		return (-1.99803137524475E+02 * x + 2.71497008797383E+02) * x + 3.42106616941255E+01;
	}
}

vec3 IDL_CB_RdPu_colormap(float x) {
	float r = clamp(colormap_red(x) / 255.0, 0.0, 1.0);
	float g = clamp(colormap_green(x) / 255.0, 0.0, 1.0);
	float b = clamp(colormap_blue(x) / 255.0, 0.0, 1.0);
	return vec3(r, g, b);
}
