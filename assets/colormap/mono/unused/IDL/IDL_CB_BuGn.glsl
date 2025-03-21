// Source from: https://github.com/kbinani/colormap-shaders
float colormap_red(float x) {
	if (x < 0.2494658606560582) {
		return (-1.98833893257484E+02 * x - 1.24389159093545E+02) * x + 2.46504720279718E+02;
	} else {
		return (((-1.85732147540803E+03 * x + 3.95435649372523E+03) * x - 2.78121710759105E+03) * x + 3.94042660425286E+02) * x + 2.23685415320351E+02;
	}
}

float colormap_green(float x) {
	if (x < 0.2248314628132282) {
		return -6.43489926739916E+01 * x + 2.52449038461538E+02;
	} else {
		return ((-5.64618971208984E+01 * x - 2.68370957359183E+01) * x - 1.13001580194466E+02) * x + 2.65385956392887E+02;
	}
}

float colormap_blue(float x) {
	if (x < 0.8) {
		return ((((1.59308890502154E+03 * x - 2.88662249445915E+03) * x + 2.00432779052853E+03) * x - 9.47781545884907E+02) * x + 5.68068034974858E+01) * x + 2.51926935643853E+02;
	} else {
		return ((-4.08819825327256E+03 * x + 1.13496840066923E+04) * x - 1.06254795336147E+04) * x + 3.39092424595566E+03;
	}
}

vec3 IDL_CB_BuGn_colormap(float x) {
	float r = clamp(colormap_red(x) / 255.0, 0.0, 1.0);
	float g = clamp(colormap_green(x) / 255.0, 0.0, 1.0);
	float b = clamp(colormap_blue(x) / 255.0, 0.0, 1.0);
	return vec3(r, g, b);
}
