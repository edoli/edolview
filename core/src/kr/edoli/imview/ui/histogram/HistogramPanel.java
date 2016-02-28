package kr.edoli.imview.ui.histogram;

import com.badlogic.gdx.scenes.scene2d.Actor;
import com.badlogic.gdx.scenes.scene2d.ui.Button;
import com.badlogic.gdx.scenes.scene2d.ui.Table;
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener;
import kr.edoli.imview.ui.UIFactory;
import kr.edoli.imview.ui.drawable.ColorDrawable;
import kr.edoli.imview.ui.res.Colors;

/**
 * Created by daniel on 16. 2. 27.
 */
public class HistogramPanel extends Table {
    public HistogramPanel() {
        final HistogramViewer histogramViewer = new ColorHistogramViewer();

        add(histogramViewer).expand().fill().row();

        Table row = new Table();

        String[] texts = new String[] {
                "red", "green", "blue"
        };

        for (int i = 0; i < histogramViewer.histogramNumber(); i++) {
            final Button checkBox = UIFactory.checkBox(texts[i]);
            final int index = i;

            checkBox.addListener(new ChangeListener() {
                @Override
                public void changed(ChangeEvent event, Actor actor) {
                    histogramViewer.setShow(index, checkBox.isChecked());
                }
            });
            checkBox.setChecked(histogramViewer.isShow(i));
            row.add(checkBox).expandX().fillX().height(20).pad(4);
        }

        add(row).expandX().fillX();

        background(new ColorDrawable(Colors.background));
    }
}
