// Source from: https://github.com/kbinani/colormap-shaders
float colormap_f(float x) {
	if (x < 0.8110263645648956) {
		return (((4.41347880412638E+03 * x - 1.18250308887283E+04) * x + 1.13092070303101E+04) * x - 4.94879610401395E+03) * x + 1.10376673162241E+03;
	} else {
		return (4.44045986053970E+02 * x - 1.34196160353499E+03) * x + 9.26518306556645E+02;
	}
}

float colormap_red(float x) {
	if (x < 0.09384074807167053) {
		return 7.56664615384615E+02 * x + 1.05870769230769E+02;
	} else if (x < 0.3011957705020905) {
		return (-2.97052932130813E+02 * x + 4.43575866219751E+02) * x + 1.37867123966178E+02;
	} else if (x < 0.3963058760920129) {
		return 8.61868131868288E+01 * x + 2.18562881562874E+02;
	} else if (x < 0.5) {
		return 2.19915384615048E+01 * x + 2.44003846153861E+02;
	} else {
		return colormap_f(x);
	}
}

float colormap_green(float x) {
	if (x < 0.09568486400411116) {
		return 2.40631111111111E+02 * x + 1.26495726495727E+00;
	} else if (x < 0.2945883673263987) {
		return 7.00971783488427E+02 * x - 4.27826773670273E+01;
	} else if (x < 0.3971604611945229) {
		return 5.31775726495706E+02 * x + 7.06051282052287E+00;
	} else if (x < 0.5) {
		return 3.64925470085438E+02 * x + 7.33268376068493E+01;
	} else {
		return colormap_f(x);
	}
}

float colormap_blue(float x) {
	if (x < 0.09892375498249567) {
		return 1.30670329670329E+02 * x + 3.12116402116402E+01;
	} else if (x < 0.1985468629735229) {
		return 3.33268034188035E+02 * x + 1.11699145299146E+01;
	} else if (x < 0.2928770209555256) {
		return 5.36891330891336E+02 * x - 2.92588522588527E+01;
	} else if (x < 0.4061551302245808) {
		return 6.60915763546766E+02 * x - 6.55827586206742E+01;
	} else if (x < 0.5) {
		return 5.64285714285700E+02 * x - 2.63359683794383E+01;
	} else {
		return colormap_f(x);
	}
}

vec3 IDL_CB_RdGy_colormap(float x) {
	float r = clamp(colormap_red(x) / 255.0, 0.0, 1.0);
	float g = clamp(colormap_green(x) / 255.0, 0.0, 1.0);
	float b = clamp(colormap_blue(x) / 255.0, 0.0, 1.0);
	return vec3(r, g, b);
}
