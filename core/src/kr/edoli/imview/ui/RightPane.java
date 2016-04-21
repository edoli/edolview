package kr.edoli.imview.ui;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Cursor;
import com.badlogic.gdx.scenes.scene2d.Actor;
import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.ui.Cell;
import com.badlogic.gdx.scenes.scene2d.ui.Table;
import com.badlogic.gdx.scenes.scene2d.ui.TextButton;
import com.badlogic.gdx.scenes.scene2d.utils.DragListener;
import com.badlogic.gdx.utils.Align;
import kr.edoli.imview.ui.drawable.ColorBorderDrawable;
import kr.edoli.imview.ui.panel.ColorPanel;
import kr.edoli.imview.ui.panel.histogram.HistogramPanel;
import kr.edoli.imview.ui.res.Colors;
import kr.edoli.imview.ui.res.FontAwesomes;
import kr.edoli.imview.ui.util.UIFactory;

/**
 * Created by daniel on 16. 2. 28.
 */
public class RightPane extends Table {

    private Table contentTable = new Table();
    private TextButton foldButton = UIFactory.iconButton(FontAwesomes.FaEllipsisV);
    private boolean isFold;
    private Cell<?> contentCell;

    private float defaultWidth = 320;
    private float minWidth = 240;

    public RightPane() {

        add(foldButton).expandY().fillY().width(12);


        contentTable.add(new HistogramPanel()).height(300).expandX().fillX().align(Align.top);
        contentTable.row();
        contentTable.add(new ColorPanel()).height(48).expandX().fillX().align(Align.top);
        contentTable.add().expandY();

        contentCell = add(contentTable).expand().fill().width(defaultWidth);

        background(new ColorBorderDrawable(Colors.background, Colors.border));

        setFold(isFold);

        foldButton.addListener(new DragListener() {
            private float offsetX;
            private float offsetY;

            private boolean isDown;
            private boolean isOver;

            @Override
            public void enter(InputEvent event, float x, float y, int pointer, Actor fromActor) {
                isOver = true;
                Gdx.graphics.setSystemCursor(Cursor.SystemCursor.Hand);

                super.enter(event, x, y, pointer, fromActor);
            }

            @Override
            public void exit(InputEvent event, float x, float y, int pointer, Actor toActor) {
                isOver = false;
                if (!isDown) {
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.Arrow);
                }

                super.exit(event, x, y, pointer, toActor);
            }

            @Override
            public boolean touchDown(InputEvent event, float x, float y, int pointer, int button) {
                offsetX = x;
                offsetY = y;
                isDown = true;
                return super.touchDown(event, x, y, pointer, button);
            }

            @Override
            public void touchUp(InputEvent event, float x, float y, int pointer, int button) {
                if (!isOver) {
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.Arrow);
                }
                isDown = false;
                super.touchUp(event, x, y, pointer, button);
            }

            @Override
            public void drag(InputEvent event, float x, float y, int pointer) {
                float width = contentCell.getActorWidth() - x + offsetX;
                if (width > 500) {
                    width = 500;
                }
                if (width < minWidth) {
                    setFold(true);
                }

                if (!isFold) {
                    contentCell.width(width);
                    invalidateHierarchy();
                } else if (x < -minWidth) {
                    setFold(false);
                }
                super.drag(event, x, y, pointer);
            }
        });
    }

    private void setFold(boolean isFold) {
        this.isFold = isFold;
        if (isFold) {
            contentTable.setVisible(false);
            contentCell.width(0);

            invalidateHierarchy();
        } else {
            contentTable.setVisible(true);

            invalidateHierarchy();
        }
    }
}
