package kr.edoli.imview;

/**
 * Created by 석준 on 2016-02-06.
 */
public class Utils {
    public static float clamp(float value, float min, float max) {
        if (value > max) {
            value = max;
        } else if (value < min) {
            value = min;
        }

        return value;
    }
}
