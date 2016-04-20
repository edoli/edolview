package kr.edoli.imview.ui.panel.histogram;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.GL30;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;
import com.badlogic.gdx.utils.Array;
import kr.edoli.imview.ui.res.Colors;
import kr.edoli.imview.ui.res.Textures;
import kr.edoli.imview.ui.util.BatchUtils;
import kr.edoli.imview.util.Histogram;
import lombok.AllArgsConstructor;
import lombok.Data;

/**
 * Created by daniel on 16. 2. 27.
 */
public class HistogramViewer extends Widget {
    private Array<HistogramGraph> histogramGraphs = new Array<HistogramGraph>();

    @Override
    public void draw(Batch batch, float parentAlpha) {
        super.draw(batch, parentAlpha);

        float x = getX();
        float y = getY();

        Color preColor = batch.getColor();

        batch.setColor(Colors.darkBackground);
        batch.draw(Textures.White, getX(), getY(), getWidth(), getHeight());


        batch.setBlendFunction(GL30.GL_SRC_ALPHA, GL30.GL_ONE);
        Color tmpColor = new Color();

        float preX;
        float preY;

        for (HistogramGraph histogramGraph : histogramGraphs) {
            if (!histogramGraph.isShow()) {
                continue;
            }

            Histogram histogram = histogramGraph.getHistogram();
            tmpColor.set(histogramGraph.getColor());


            tmpColor.a = 0.5f;
            batch.setColor(tmpColor);


            preX = x;
            preY = y;

            float delta = getWidth() / histogram.getNumber();
            for (int i = 0; i < histogram.getNumber(); i++) {
                float value = (float) histogram.getFreq(i) / histogram.getMaxFreq();
                float currentX = x + i * delta;
                float currentY = y + value * getHeight();
                BatchUtils.drawQuad(batch, preX, y, preX, preY, currentX, currentY, currentX, y);
                preX = currentX;
                preY = currentY;
            }
        }

        for (HistogramGraph histogramGraph : histogramGraphs) {
            if (!histogramGraph.isShow()) {
                continue;
            }

            Histogram histogram = histogramGraph.getHistogram();
            tmpColor.set(histogramGraph.getColor());

            tmpColor.a = 0.5f;
            batch.setColor(tmpColor);

            preX = x;
            preY = y;

            float delta = getWidth() / histogram.getNumber();
            for (int i = 0; i < histogram.getNumber(); i++) {
                float value = (float) histogram.getFreq(i) / histogram.getMaxFreq();
                float currentX = x + i * delta;
                float currentY = y + value * getHeight();
                BatchUtils.drawLine(batch, preX, preY, currentX, currentY, 1);
                preX = currentX;
                preY = currentY;
            }
        }

        batch.setColor(preColor);
        batch.setBlendFunction(GL30.GL_SRC_ALPHA, GL30.GL_ONE_MINUS_SRC_ALPHA);
    }

    public Histogram addHistogram(Histogram histogram, Color histogramColor) {
        histogramGraphs.add(new HistogramGraph(histogram, histogramColor, true));

        return histogram;
    }

    public int histogramNumber() {
        return histogramGraphs.size;
    }

    public void setShow(int index, boolean value) {
        histogramGraphs.get(index).show = value;
    }

    public void show(int index) {
        histogramGraphs.get(index).show = true;
    }

    public void hide(int index) {
        histogramGraphs.get(index).show = false;
    }

    public boolean isShow(int index) {
        return histogramGraphs.get(index).show;
    }

    @Data
    @AllArgsConstructor
    private static class HistogramGraph {
        private Histogram histogram;
        private Color color;
        private boolean show;
    }
}
