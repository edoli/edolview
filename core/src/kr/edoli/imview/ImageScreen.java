package kr.edoli.imview;

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
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.ArrayUtils;
import org.lwjgl.glfw.GLFW;
import org.lwjgl.glfw.GLFWDropCallback;

import java.io.File;
import java.util.Arrays;

/**
 * Created by 석준 on 2016-02-06.
 */
public class ImageScreen implements Screen {

    private Stage stage = new Stage(new ScreenViewport());
    private String imagePath;

    private ImageViewer imageViewer;
    private PanningView panningView;

    public ImageScreen() {

        Table table = new Table();
        Table overlayTable = new Table();

        imagePath = ImView.imagePath;

        imageViewer = new ImageViewer();
        imageViewer.setImage(imagePath);
        imageViewer.setFillParent(true);

        panningView = new PanningView(imageViewer);

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


        Array<Lwjgl3Window> windows = (Array<Lwjgl3Window>) Utils.getPrivate(Gdx.app, "windows");

        long windowHandle = (Long) Utils.getPrivate(windows.get(0), "windowHandle");

        GLFW.glfwSetDropCallback(windowHandle, new GLFWDropCallback() {
            @Override
            public void invoke(long window, int count, long names) {
                String path = getNames(1, names)[0];

                changeImage(path);
            }
        });

        stage.addActor(table);
        stage.addActor(overlayTable);

        stage.setScrollFocus(panningView);

        stage.addListener(new InputListener() {
            @Override
            public boolean keyDown(InputEvent event, int keycode) {

                if (keycode == Input.Keys.RIGHT || keycode == Input.Keys.LEFT) {
                    File file = new File(imagePath);
                    String[] fileNames = file.getParentFile().list();
                    String fileName = file.getName();
                    Arrays.sort(fileNames);

                    int index = ArrayUtils.indexOf(fileNames, fileName);
                    String path;

                    int modifier = keycode == Input.Keys.RIGHT ? 1 : -1;

                    do {
                        index = index + modifier;

                        if (index == fileNames.length) {
                            index = 0;
                        }
                        if (index == -1) {
                            index = fileNames.length - 1;
                        }

                        path = file.getParent() + "/" + fileNames[index];
                    } while (!isImage(path));

                    changeImage(path);

                    return true;
                }

                return super.keyDown(event, keycode);
            }
        });
    }

    private boolean isImage(String path) {
        String ext = FilenameUtils.getExtension(path);
        String[] exts = new String[] {"png", "PNG", "jpg", "JPG"};

        return ArrayUtils.contains(exts, ext);
    }

    private void changeImage(String path) {
        if (!isImage(path)) {
            return;
        }

        imagePath = path;

        imageViewer.setImage(imagePath);
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
