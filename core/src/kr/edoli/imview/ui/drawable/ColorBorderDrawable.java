package kr.edoli.imview.ui.drawable;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable;
import kr.edoli.imview.ui.res.Textures;

/**
 * Created by 석준 on 2016-02-06.
 */
public class ColorBorderDrawable extends BaseDrawable {

    private Color color;
    private Color borderColor;

    public ColorBorderDrawable(Color color, Color borderColor) {
        this.color = color;
        this.borderColor = borderColor;
    }

    @Override
    public void draw(Batch batch, float x, float y, float width, float height) {
        Color preColor = batch.getColor();

        batch.setColor(color);
        batch.draw(Textures.White, x, y, width, height);

        batch.setColor(borderColor);
        batch.draw(Textures.White, x, y, width, 1);
        batch.draw(Textures.White, x, y + height - 1, width, 1);
        batch.draw(Textures.White, x, y, 1, height);
        batch.draw(Textures.White, x + width - 1, y, 1, height);

        batch.setColor(preColor);

        super.draw(batch, x, y, width, height);
    }
}
