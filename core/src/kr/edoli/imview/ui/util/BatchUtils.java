package kr.edoli.imview.ui.util;

import com.badlogic.gdx.graphics.g2d.Batch;
import kr.edoli.imview.ui.res.Textures;

/**
 * Created by daniel on 16. 2. 28.
 */
public class BatchUtils {
    private static float[] verticies = new float[20];

    public static void drawRect(Batch batch, float x, float y, float width, float height) {
        drawQuad(batch, x, y, x + width, y, x + width, y + height, x, y + height);
    }

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

    public static void drawLine(Batch batch, float x1, float y1, float x2, float y2, float lineWidth) {
        float dx = x2 - x1;
        float dy = y2 - y1;

        float len = (float) Math.hypot(dx, dy);

        lineWidth /= 2;
        float ndx = (dx / len) * lineWidth;
        float ndy = (dy / len) * lineWidth;

        x1 -= ndx;
        x2 += ndx;
        y1 -= ndy;
        y2 += ndy;

        drawQuad(batch,
                x1 + ndy, y1 - ndx,
                x1 - ndy, y1 + ndx,
                x2 - ndy, y2 + ndx,
                x2 + ndy, y2 - ndx);
    }

    public static void drawRectBorder(Batch batch, float x, float y, float width, float height, float lineWidth) {
        drawBorder(batch, x, y, x + width, y, x + width, y + height, x, y + height, lineWidth);
    }

    public static void drawBorder(Batch batch, float x1, float y1, float x2, float y2
            , float x3, float y3, float x4, float y4, float lineWidth) {
        drawLine(batch, x1, y1, x2, y2, lineWidth);
        drawLine(batch, x2, y2, x3, y3, lineWidth);
        drawLine(batch, x3, y3, x4, y4, lineWidth);
        drawLine(batch, x4, y4, x1, y1, lineWidth);
    }
}
