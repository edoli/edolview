package kr.edoli.imview.ui;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;
import kr.edoli.imview.ui.res.Textures;

/**
 * Created by daniel on 16. 2. 27.
 */
public class ColorWidget extends Widget {
    private Color color;

    public ColorWidget(Color color) {
        this.color = color;
    }

    @Override
    public void draw(Batch batch, float parentAlpha) {
        super.draw(batch, parentAlpha);

        Color preColor = batch.getColor();

        batch.setColor(color);
        batch.draw(Textures.White, getX(), getY(), getWidth(), getHeight());

        batch.setColor(preColor);
    }
}
