package kr.edoli.imview.ui;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.math.Vector2;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;
import kr.edoli.imview.Context;
import kr.edoli.imview.ui.PanningView;
import kr.edoli.imview.ui.res.Colors;
import kr.edoli.imview.ui.res.Textures;

/**
 * Created by 석준 on 2016-02-06.
 */
public class DataView extends Widget {

    private BitmapFont font = new BitmapFont();

    @Override
    public void draw(Batch batch, float parentAlpha) {
        super.draw(batch, parentAlpha);

        batch.setColor(Colors.background);
        batch.draw(Textures.White, getX(), getY(), getWidth(), getHeight());

        batch.setColor(Color.WHITE);
        Vector2 mousePosOnImage = Context.getMousePosOnImage();

        float pixelX = mousePosOnImage.x;
        float pixelY = mousePosOnImage.y;
        font.draw(batch, String.format("%.02f, %.02f", pixelX, pixelY), getX() + 12, getY() + 22);
    }
}
