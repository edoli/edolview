package kr.edoli.imview.ui.util;

import com.badlogic.gdx.graphics.g2d.Batch;
import kr.edoli.imview.ui.res.Textures;

/**
 * Created by daniel on 16. 2. 28.
 */
public class BatchUtils {
    private static float[] verticies = new float[20];

    public static void drawQuad(Batch batch, float x1, float y1, float x2, float y2
            , float x3, float y3, float x4, float y4) {
        float color = batch.getPackedColor();

        int idx = 0;

        verticies[idx++] = x1;
        verticies[idx++] = y1;
        verticies[idx++] = color;
        verticies[idx++] = 0;
        verticies[idx++] = 0;

        verticies[idx++] = x2;
        verticies[idx++] = y2;
        verticies[idx++] = color;
        verticies[idx++] = 1;
        verticies[idx++] = 0;

        verticies[idx++] = x3;
        verticies[idx++] = y3;
        verticies[idx++] = color;
        verticies[idx++] = 0;
        verticies[idx++] = 1;

        verticies[idx++] = x4;
        verticies[idx++] = y4;
        verticies[idx++] = color;
        verticies[idx++] = 1;
        verticies[idx++] = 1;

        batch.draw(Textures.White, verticies, 0, 20);
    }

    public static void drawLine(Batch batch, float x1, float y1, float x2, float y2) {

        float color = batch.getPackedColor();

        int idx = 0;

        verticies[idx++] = x1;
        verticies[idx++] = y1;
        verticies[idx++] = color;
        verticies[idx++] = 0;
        verticies[idx++] = 0;

        verticies[idx++] = x1 + 1;
        verticies[idx++] = y1 + 1;
        verticies[idx++] = color;
        verticies[idx++] = 1;
        verticies[idx++] = 0;

        verticies[idx++] = x2;
        verticies[idx++] = y2;
        verticies[idx++] = color;
        verticies[idx++] = 0;
        verticies[idx++] = 1;

        verticies[idx++] = x2 + 1;
        verticies[idx++] = y2 + 1;
        verticies[idx++] = color;
        verticies[idx++] = 1;
        verticies[idx++] = 1;

        batch.draw(Textures.White, verticies, 0, 20);
    }
}
