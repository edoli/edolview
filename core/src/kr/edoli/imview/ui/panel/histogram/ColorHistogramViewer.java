package kr.edoli.imview.ui.panel.histogram;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.Pixmap;
import com.badlogic.gdx.math.Rectangle;
import kr.edoli.imview.Context;
import kr.edoli.imview.util.Histogram;
import kr.edoli.imview.value.ValueListener;


/**
 * Created by daniel on 16. 2. 27.
 */
public class ColorHistogramViewer extends HistogramViewer {
    private Histogram redHistogram;
    private Histogram blueHistogram;
    private Histogram greenHistogram;

    public ColorHistogramViewer() {
        redHistogram = addHistogram(new Histogram(255), Color.RED);
        greenHistogram = addHistogram(new Histogram(255), Color.GREEN);
        blueHistogram = addHistogram(new Histogram(255), Color.BLUE);

        Context.selectedRegionOnImage.addListener(new ValueListener<Rectangle>() {
            @Override
            public void change(Rectangle rect) {
                setRegion(Context.currentImage.get(), rect);
            }
        });
    }

    private void setRegion(Pixmap pixmap, Rectangle rectangle) {

        redHistogram.clear();
        greenHistogram.clear();
        blueHistogram.clear();

        int left = 0;
        int top = 0;
        int right = pixmap.getWidth();
        int bottom = pixmap.getHeight();

        if (rectangle != null) {
            left = (int) rectangle.x;
            top = (int) rectangle.y;
            right = (int) rectangle.width + left;
            bottom = (int) rectangle.height + top;

        }



        int color, r, g, b;
        for (int x = left; x < right; x++) {
            for (int y = top; y < bottom; y++) {
                color = pixmap.getPixel(x, y);

                r = (color >> 24) & 0xFF;
                g = (color >> 16) & 0xFF;
                b = (color >> 8) & 0xFF;

                if (r > 0) {
                    redHistogram.addDataPoint(r - 1);
                }
                if (g > 0) {
                    greenHistogram.addDataPoint(g - 1);
                }
                if (b > 0) {
                    blueHistogram.addDataPoint(b - 1);
                }
            }
        }
    }


}
