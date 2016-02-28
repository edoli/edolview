package kr.edoli.imview.ui;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.Input;
import com.badlogic.gdx.Screen;
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Window;
import com.badlogic.gdx.graphics.GL20;
import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.InputListener;
import com.badlogic.gdx.scenes.scene2d.Stage;
import com.badlogic.gdx.scenes.scene2d.ui.Table;
import com.badlogic.gdx.utils.Array;
import com.badlogic.gdx.utils.viewport.ScreenViewport;
import kr.edoli.imview.Context;
import kr.edoli.imview.ui.res.Colors;
import kr.edoli.imview.util.ImageFileUtils;
import kr.edoli.imview.util.Utils;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWDropCallback;

/**
 * Created by 석준 on 2016-02-06.
 */
public class MainScreen implements Screen {

    private Stage stage = new Stage(new ScreenViewport());

    private ImageViewer imageViewer;
    private PanningView panningView;

    public MainScreen() {

        ColorActor background = new ColorActor(Colors.background);
        background.setSize(stage.getWidth(), stage.getHeight());

        Table table = new Table();
        Table overlayTable = new Table();

        imageViewer = new ImageViewer();
        imageViewer.setImage(Context.imagePath.get());
        imageViewer.setFillParent(true);

        panningView = new PanningView(imageViewer);

        table.add(panningView).expand().fill();
        table.setFillParent(true);


        DataView dataView = new DataView();

        Table rightPanel = new RightPanel();

        overlayTable.add().expandY().fillY().width(196);
        overlayTable.add().expand();
        overlayTable.add(rightPanel).expandY().fillY().row();
        overlayTable.add(dataView).colspan(3).expandX().fillX().height(32);
        overlayTable.setFillParent(true);
//
//        table.debug();
//        overlayTable.debug();


        Array<Lwjgl3Window> windows = (Array<Lwjgl3Window>) Utils.getPrivate(Gdx.app, "windows");

        long windowHandle = (Long) Utils.getPrivate(windows.get(0), "windowHandle");

        GLFW.glfwSetDropCallback(windowHandle, new GLFWDropCallback() {
            @Override
            public void invoke(long window, int count, long names) {
                String path = getNames(1, names)[0];

                changeImage(path);
            }
        });

        stage.addActor(background);
        stage.addActor(table);
        stage.addActor(overlayTable);

        stage.setScrollFocus(panningView);

        stage.addListener(new InputListener() {
            @Override
            public boolean keyDown(InputEvent event, int keycode) {

                if (keycode == Input.Keys.RIGHT || keycode == Input.Keys.LEFT) {
                    int modifier = keycode == Input.Keys.RIGHT ? 1 : -1;

                    changeImage(ImageFileUtils.siblingFile(Context.imagePath.get(), modifier).getPath());

                    return true;
                }

                return super.keyDown(event, keycode);
            }
        });

        Context.stage = stage;
        Context.panningView = panningView;
    }

    private void changeImage(String path) {
        if (!ImageFileUtils.isImage(path)) {
            return;
        }

        Context.imagePath.set(path);

        imageViewer.setImage(path);
        panningView.reset();
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
        stage.getViewport().update(width, height, true);
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
