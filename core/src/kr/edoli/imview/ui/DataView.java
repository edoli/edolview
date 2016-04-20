package kr.edoli.imview.ui;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.math.Rectangle;
import com.badlogic.gdx.math.Vector2;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;
import com.badlogic.gdx.scenes.scene2d.utils.Drawable;
import kr.edoli.imview.Context;
import kr.edoli.imview.ui.drawable.ColorBorderDrawable;
import kr.edoli.imview.ui.res.Colors;
import kr.edoli.imview.ui.util.UIFactory;

/**
 * Created by 석준 on 2016-02-06.
 */
public class DataView extends Widget {

    private BitmapFont font = UIFactory.getFont();
    private Drawable background = new ColorBorderDrawable(Colors.background, Colors.border);

    @Override
    public void draw(Batch batch, float parentAlpha) {
        super.draw(batch, parentAlpha);

        batch.setColor(Colors.background);
        background.draw(batch, getX(), getY(), getWidth(), getHeight());

        batch.setColor(Color.WHITE);
        Vector2 mousePosOnImage = Context.getMousePosOnImage();

        float pixelX = mousePosOnImage.x;
        float pixelY = mousePosOnImage.y;
        font.draw(batch, String.format("%.02f, %.02f", pixelX, pixelY), getX() + 12, getY() + 22);

        Rectangle selectedRegion = Context.selectedRegionOnImage.get();
        if (selectedRegion != null) {
            font.draw(batch, String.format("%d, %d, %d, %d",
                    (int) selectedRegion.x,
                    (int) selectedRegion.y,
                    (int) selectedRegion.width,
                    (int) selectedRegion.height), getX() + 256, getY() + 22);
        }

        int zoom = (int) (Context.zoom.get() * 100);
        font.draw(batch, String.format("%d %%", zoom), getX() + getWidth() - 64, getY() + 22);
    }
}
