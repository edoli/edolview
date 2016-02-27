package kr.edoli.imview.util;

import java.lang.reflect.Field;

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

    public static Object getPrivate(Object obj, String fieldName) {

        Field field = null;
        try {
            field = obj.getClass().getDeclaredField(fieldName);
            field.setAccessible(true);
            return field.get(obj);

        } catch (NoSuchFieldException e) {
            e.printStackTrace();
        } catch (IllegalAccessException e) {
            e.printStackTrace();
        }

        return null;
    }
}
