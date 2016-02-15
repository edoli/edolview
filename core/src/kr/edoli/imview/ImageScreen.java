package kr.edoli.imview;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.Screen;
import com.badlogic.gdx.graphics.GL20;
import com.badlogic.gdx.scenes.scene2d.Stage;
import com.badlogic.gdx.scenes.scene2d.ui.Table;
import com.badlogic.gdx.utils.viewport.ScreenViewport;

/**
 * Created by 석준 on 2016-02-06.
 */
public class ImageScreen implements Screen {

    private Stage stage = new Stage(new ScreenViewport());

    public ImageScreen() {

        Table table = new Table();
        Table overlayTable = new Table();

        ImageViewer imageViewer = new ImageViewer();
        imageViewer.setImage("test.png");
        imageViewer.setFillParent(true);

        PanningView panningView = new PanningView(imageViewer);

        table.add(panningView).expand().fill();
        table.setFillParent(true);


        DataView dataView = new DataView(panningView);

        overlayTable.add().expandY().fillY().width(196);
        overlayTable.add().expand();
        overlayTable.add(new FilterView()).expandY().fillY().width(128).row();
        overlayTable.add(dataView).colspan(2).expandX().fillX().height(32);
        overlayTable.setFillParent(true);

        table.debug();
        overlayTable.debug();

        stage.addActor(table);
        stage.addActor(overlayTable);

        stage.setScrollFocus(panningView);
    }

    @Override
    public void show() {
        Gdx.input.setInputProcessor(stage);
    }

    @Override
    public void render(float delta) {
        stage.act(delta);

        Gdx.gl.glClearColor(0, 0, 0, 1);
        Gdx.gl.glClear(GL20.GL_COLOR_BUFFER_BIT);

        stage.draw();
    }

    @Override
    public void resize(int width, int height) {
        stage.getViewport().update(width, height);
    }

    @Override
    public void pause() {

    }

    @Override
    public void resume() {

    }

    @Override
    public void hide() {

    }

    @Override
    public void dispose() {

    }
}
