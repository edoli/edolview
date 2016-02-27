package kr.edoli.imview.ui;

import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;
import com.badlogic.gdx.utils.Array;
import kr.edoli.imview.ui.res.Textures;
import kr.edoli.imview.util.Histogram;

/**
 * Created by daniel on 16. 2. 27.
 */
public class HistogramViewer extends Widget {
    private Array<Histogram> histograms = new Array<Histogram>();

    @Override
    public void draw(Batch batch, float parentAlpha) {
        super.draw(batch, parentAlpha);

        float x = getX();
        float y = getY();

        for (Histogram histogram : histograms) {
            for (int i = 0; i < histogram.getNumber(); i++) {
                batch.draw(Textures.White, x + i, y, 1, histogram.getFreq(i));
            }
        }
    }
}
