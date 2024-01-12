// Source from: https://github.com/kbinani/colormap-shaders
vec3 IDL_Rainbow_18_colormap(float x) {
	float x16 = x * 16.0;
	const float s = 1.0 / 255.0;
	if (x16 < 1.0) {
		return vec3(150.0, 0.0, 150.0) * s;
	} else if (x16 < 2.0) {
		return vec3(200.0, 0.0, 200.0) * s;
	} else if (x16 < 3.0) {
		return vec3(100.0, 100.0, 150.0) * s;
	} else if (x16 < 4.0) {
		return vec3(100.0, 100.0, 200.0) * s;
	} else if (x16 < 5.0) {
		return vec3(100.0, 100.0, 255.0) * s;
	} else if (x16 < 6.0) {
		return vec3(0.0, 140.0, 0.0) * s;
	} else if (x16 < 7.0) {
		return vec3(150.0, 170.0, 0.0) *s;
	} else if (x16 < 8.0) {
		return vec3(200.0, 200.0, 0.0) * s;
	} else if (x16 < 9.0) {
		return vec3(150.0, 200.0, 0.0) * s;
	} else if (x16 < 10.0) {
		return vec3(200.0, 255.0, 120.0) * s;
	} else if (x16 < 11.0) {
		return vec3(255.0, 255.0, 0.0) * s;
	} else if (x16 < 12.0) {
		return vec3(255.0, 200.0, 0.0) * s;
	} else if (x16 < 13.0) {
		return vec3(255.0, 160.0, 0.0) * s;
	} else if (x16 < 14.0) {
		return vec3(255.0, 125.0, 0.0) * s;
	} else if (x16 < 15.0) {
		return vec3(200.0, 50.0, 100.0) * s;
	} else {
		return vec3(175.0, 50.0, 75.0) * s;
	}
}
