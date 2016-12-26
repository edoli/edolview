package kr.edoli.imview.image;

import com.badlogic.gdx.graphics.Pixmap;

import java.nio.ByteBuffer;

/**
 * Created by sjjeon on 16. 12. 20.
 */
public class JavaImageTest {
    public static Pixmap test(Pixmap pixmap) {

        ByteBuffer pixels = pixmap.getPixels();
        Pixmap nPixmap = new Pixmap(pixmap.getWidth(), pixmap.getHeight(), pixmap.getFormat());
        ByteBuffer nPixels = nPixmap.getPixels();

        int width = pixmap.getWidth();
        int height = pixmap.getHeight();
        int[] kernel = new int[]{
            -width - 1, -width, -width + 1,
                    -1, 0, 1,
                    width - 1, width, width + 1
        };

        int tx = 0;
        int ty = 0;
        for (tx = 1; tx < width - 1; tx++) {
            for (ty = 1; ty < height - 1; ty++) {
                int ind = (tx + ty * width) * 3;

                int r = 0;
                int g = 0;
                int b = 0;

                int cx = 0;
                int cy = 0;

                for (cx = -1; cx < 2; cx++) {
                    for (cy = -1; cy < 2; cy++) {
                        int pind = ind + (cx + cy * width) * 3;
                        r += pixels.get(pind);
                        g += pixels.get(pind + 1);
                        b += pixels.get(pind + 2);
                    }
                }

                nPixels.put(ind, (byte) (r / 9));
                nPixels.put(ind, (byte) (g / 9));
                nPixels.put(ind, (byte) (b / 9));
            }
        }

        return pixmap;

    }
}
