package kr.edoli.imview.ui.drawable;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable;
import kr.edoli.imview.ui.res.Textures;

/**
 * Created by 석준 on 2016-02-06.
 */
public class ColorDrawable extends BaseDrawable {

    private Color color;

    public ColorDrawable(Color color) {
        this.color = color;
    }

    @Override
    public void draw(Batch batch, float x, float y, float width, float height) {
        Color preColor = batch.getColor();

        batch.setColor(color);
        batch.draw(Textures.White, x, y, width, height);

        batch.setColor(preColor);

        super.draw(batch, x, y, width, height);
    }
}
