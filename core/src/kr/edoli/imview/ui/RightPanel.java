package kr.edoli.imview.ui;

import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.ui.Cell;
import com.badlogic.gdx.scenes.scene2d.ui.ImageButton;
import com.badlogic.gdx.scenes.scene2d.ui.Table;
import com.badlogic.gdx.scenes.scene2d.utils.DragListener;
import com.badlogic.gdx.utils.Align;
import kr.edoli.imview.ui.drawable.ColorBorderDrawable;
import kr.edoli.imview.ui.histogram.HistogramPanel;
import kr.edoli.imview.ui.res.Colors;
import kr.edoli.imview.ui.res.Drawables;

/**
 * Created by daniel on 16. 2. 28.
 */
public class RightPanel extends Table {

    private Table contentTable = new Table();
    private ImageButton foldButton = UIFactory.iconButton(null);
    private Cell<?> contentCell;

    public RightPanel() {

        add(foldButton).expandY().fillY().width(12);


        contentTable.add(new HistogramPanel()).height(300).expand().fillX().align(Align.top);

        contentCell = add(contentTable).expand().fill().width(256);

        background(new ColorBorderDrawable(Colors.background, Colors.border));

        /*
        foldButton.addListener(new ClickListener() {
            @Override
            public void clicked(InputEvent event, float x, float y) {
                super.clicked(event, x, y);

                setFold(foldButton.isChecked());
            }
        });

        setFold(foldButton.isChecked());
        */

        foldButton.addListener(new DragListener() {


            @Override
            public void drag(InputEvent event, float x, float y, int pointer) {
                contentCell.width(contentCell.getActorWidth() - x);
                invalidateHierarchy();
                super.drag(event, x, y, pointer);
            }
        });
    }

    private void setFold(boolean isFold) {
        if (isFold) {
            contentTable.setVisible(false);
            contentCell.width(0);

            foldButton.getStyle().imageUp = Drawables.leftArrow;

            invalidateHierarchy();


        } else {
            contentTable.setVisible(true);
            contentCell.width(256);

            foldButton.getStyle().imageUp = Drawables.rightArrow;

            invalidateHierarchy();
        }
    }
}
