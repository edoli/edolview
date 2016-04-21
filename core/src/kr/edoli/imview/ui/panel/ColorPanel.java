package kr.edoli.imview.ui.panel;

import com.badlogic.gdx.graphics.Pixmap;
import com.badlogic.gdx.math.Rectangle;
import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.ui.Button;
import com.badlogic.gdx.scenes.scene2d.ui.Label;
import com.badlogic.gdx.scenes.scene2d.ui.TextButton;
import com.badlogic.gdx.scenes.scene2d.ui.TextField;
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener;
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils;
import com.badlogic.gdx.utils.Align;
import kr.edoli.imview.Context;
import kr.edoli.imview.ui.OptionList;
import kr.edoli.imview.ui.res.FontAwesomes;
import kr.edoli.imview.ui.util.UIFactory;
import kr.edoli.imview.util.Clipboard;
import kr.edoli.imview.value.ValueListener;

/**
 * Created by sjjeon on 16. 4. 20.
 */
public class ColorPanel extends Panel {

    enum ColorType {
        RGB {
            @Override
            String colorToString(float r, float g, float b) {
                return String.format("%d, %d, %d", (int) r, (int) g, (int) b);
            }
        },
        RGBFloat {
            @Override
            String colorToString(float r, float g, float b) {
                return String.format("%.2f, %.2f, %.2f", r / 255, g / 255, b / 255);
            }
        },
        Hex {
            @Override
            String colorToString(float r, float g, float b) {
                return String.format("#%02X%02X%02X", (int) r, (int) g, (int) b);
            }
        };

        abstract String colorToString(float r, float g, float b);
    }

    private ColorType colorType;
    private Label colorLabel;
    private TextButton colorCopyButton;
    private OptionList optionList;

    private float r;
    private float g;
    private float b;

    public ColorPanel() {
        colorLabel = UIFactory.label("");
        colorCopyButton = UIFactory.iconButton(FontAwesomes.FaCopy);
        optionList = new OptionList(new String[] {
                "RGB",
                "Float",
                "Hex"
        });
        optionList.setOptionChangedListener(new OptionList.OptionChangedListener() {
            @Override
            public void changed(Button option, int id) {
                if (id == 0) {
                    colorType = ColorType.RGB;
                } else if (id == 1) {
                    colorType = ColorType.RGBFloat;
                } else if (id == 2) {
                    colorType = ColorType.Hex;
                }
                reFormatText();
            }
        });

        colorLabel.getStyle().font.getData().markupEnabled = true;

        add(UIFactory.label("Color:")).pad(0, 8, 0, 8);
        add(colorLabel).expandX().align(Align.left);
        add(colorCopyButton).size(32);
        row();
        add(optionList).colspan(3).expandX().fillX();


        colorCopyButton.addListener(new ClickListener() {
            @Override
            public void clicked(InputEvent event, float x, float y) {
                Clipboard.copy(colorLabel.getText().toString());
            }
        });

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

        this.r = r / count;
        this.g = g / count;
        this.b = b / count;

        reFormatText();
    }

    private void reFormatText() {
        colorLabel.setText(colorType.colorToString(r, g, b));
    }
}
