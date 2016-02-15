package kr.edoli.imview;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;

/**
 * Created by 석준 on 2016-02-06.
 */
public class DataView extends Widget {

    private PanningView panningView;
    private BitmapFont font = new BitmapFont();

    public DataView(PanningView panningView) {
        this.panningView = panningView;
    }

    @Override
    public void draw(Batch batch, float parentAlpha) {
        super.draw(batch, parentAlpha);

        batch.setColor(0, 0, 0, 0.75f);
        batch.draw(Textures.White, getX(), getY(), getWidth(), getHeight());

        batch.setColor(Color.WHITE);
        font.draw(batch, String.format("%.02f, %.02f", panningView.getMouseXOnImage(), panningView.getMouseYOnImage()), getX() + 12, getY() + 22);
    }
}
