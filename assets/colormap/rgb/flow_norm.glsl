// ABS_MAX

// int RY = 15;
// int YG = 6;
// int GC = 4;
// int CB = 11;
// int BM = 13;
// int MR = 6;

float RY = 15.;
float YG = 21.;
float GC = 25.;
float CB = 36.;
float BM = 49.;
float MR = 55.;

vec3 flow_norm_colormap(vec3 t) {
    float fx = t.x * float(width);
    float fy = t.y * float(height);

    float flow_norm = sqrt(fx * fx + fy * fy);
    float angle = atan(-fy, -fx) / PI;
    float fk = (angle + 1.) / 2. * 55.;

    float r = 0.;
    float g = 0.;
    float b = 0.;

    if (fk < RY) {
        r = 1.;
        g = fk / RY;
    } else if (fk < YG) {
        r = 1. - (fk - RY) / (YG - RY);
        g = 1.;
    } else if (fk < GC) {
        g = 1.;
        b = (fk - YG) / (GC - YG);
    } else if (fk < CB) {
        g = 1. - (fk - GC) / (CB - GC);
        b = 1.;
    } else if (fk < BM) {
        b = 1.;
        r = (fk - CB) / (BM - CB);
    } else if (fk <= MR) {
        b = 1. - (fk - BM) / (MR - BM);
        r = 1.;
    }

    vec3 color = vec3(r, g, b);
    color = 1. - flow_norm * (1. - color);

    return color;
}