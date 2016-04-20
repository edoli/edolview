package kr.edoli.imview.ui.panel;

import com.badlogic.gdx.graphics.Pixmap;
import com.badlogic.gdx.math.Rectangle;
import com.badlogic.gdx.scenes.scene2d.ui.Label;
import com.badlogic.gdx.scenes.scene2d.ui.TextField;
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils;
import kr.edoli.imview.Context;
import kr.edoli.imview.ui.util.UIFactory;
import kr.edoli.imview.value.ValueListener;

/**
 * Created by sjjeon on 16. 4. 20.
 */
public class ColorPanel extends Panel {

    private TextField colorLabel;

    public ColorPanel() {
        colorLabel = UIFactory.textField("");
        add(colorLabel);

        Context.selectedRegionOnImage.addListener(new ValueListener<Rectangle>() {
            @Override
            public void change(Rectangle rect) {
                setRegion(Context.currentImage.get(), rect);
            }
        });
    }

    private void setRegion(Pixmap pixmap, Rectangle rectangle) {

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

        int color;
        int r = 0, g = 0, b = 0;
        for (int x = left; x < right; x++) {
            for (int y = top; y < bottom; y++) {
                color = pixmap.getPixel(x, y);

                r += (color >> 24) & 0xFF;
                g += (color >> 16) & 0xFF;
                b += (color >> 8) & 0xFF;
            }
        }
        int count = (right - left) * (bottom - top);

        r = r / count;
        g = g / count;
        b = b / count;

        colorLabel.setText(String.format("%d, %d, %d", r, g, b));
    }
}
