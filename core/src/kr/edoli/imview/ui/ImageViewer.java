package kr.edoli.imview.ui;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.files.FileHandle;
import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.Pixmap;
import com.badlogic.gdx.graphics.Texture;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.graphics.g2d.TextureRegion;
import com.badlogic.gdx.math.Rectangle;
import com.badlogic.gdx.math.Vector2;
import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.InputListener;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils;
import kr.edoli.imview.Bus;
import kr.edoli.imview.Context;
import kr.edoli.imview.event.FilterMessage;
import kr.edoli.imview.event.MagnifyMessage;
import kr.edoli.imview.ui.res.Colors;
import kr.edoli.imview.ui.res.Textures;
import kr.edoli.imview.util.Clipboard;
import net.engio.mbassy.listener.Handler;

/**
 * Created by 석준 on 2016-02-06.
 */
public class ImageViewer extends Widget {
    private TextureRegion region;

    private Pixmap pixmap;
    private Pixmap filteredPixmap;

    private Rectangle selectedRegionOnView = new Rectangle();

    enum DragMode {
        Magnify, Analysis
    }

    public ImageViewer() {
        Bus.subscribe(this);

        addListener(new InputListener() {

            private DragMode dragMode;

            private Vector2 startPos = new Vector2();
            private Vector2 toPos = new Vector2();

            @Override
            public boolean touchDown(InputEvent event, float x, float y, int pointer, int button) {
                if (UIUtils.ctrl()) {
                    startPos.set(x, y);
                    dragMode = DragMode.Magnify;
                    return true;
                }
                if (UIUtils.alt()) {
                    startPos.set(x, y);
                    dragMode = DragMode.Analysis;
                    return true;
                }
                return false;
            }

            @Override
            public void touchDragged(InputEvent event, float x, float y, int pointer) {
                toPos.set(x, y);

                float lowX = Math.min(startPos.x, toPos.x);
                float highX = Math.max(startPos.x, toPos.x);

                float lowY = Math.min(startPos.y, toPos.y);
                float highY = Math.max(startPos.y, toPos.y);

                lowX = (float) Math.floor(lowX);
                lowY = (float) Math.floor(lowY);

                highX = (float) Math.ceil(highX);
                highY = (float) Math.ceil(highY);

                selectedRegionOnView.set(lowX, lowY, highX - lowX, highY - lowY);
            }

            @Override
            public void touchUp(InputEvent event, float x, float y, int pointer, int button) {
                switch (dragMode) {
                    case Magnify:
                        Bus.publish(new MagnifyMessage(selectedRegionOnView));
                        selectedRegionOnView.set(0, 0, 0, 0);
                        break;
                    case Analysis:
                        Context.selectedRegionOnImage.get().set(
                                selectedRegionOnView.x,
                                Context.currentImage.get().getHeight() - selectedRegionOnView.y,
                                selectedRegionOnView.width,
                                selectedRegionOnView.height
                        );
                        Context.selectedRegionOnImage.forceUpdate();
                        break;
                }
            }
        });
    }

    public void setImage(String path) {
        if (path == null) {
            return;
        }

        FileHandle fileHandle = Gdx.files.internal(path);

        if (!fileHandle.exists()) {
            return;
        }

        if (pixmap != null) {
            pixmap.dispose();
        }

        pixmap = new Pixmap(fileHandle);
        region = new TextureRegion(new Texture(pixmap));

        Context.currentImage.set(pixmap);

        region.getTexture().setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Nearest);

        setSize(region.getRegionWidth(), region.getRegionHeight());

        Clipboard.copy(pixmap, 100, 100, 500, 500);
    }

    @Override
    public void act(float delta) {
        super.act(delta);
    }

    @Override
    public void draw(Batch batch, float parentAlpha) {
        if (region != null) {
            batch.draw(region, getX(), getY(), getOriginX(), getOriginY(), getWidth(), getHeight(), getScaleX(), getScaleY(), getRotation());
        }


        Color preColor = batch.getColor();
        batch.setColor(Colors.overlayRegion);
        batch.draw(Textures.White,
                selectedRegionOnView.x * getScaleX() + getX(),
                selectedRegionOnView.y * getScaleY() + getY(),
                selectedRegionOnView.width * getScaleX(),
                selectedRegionOnView.height * getScaleY());
        batch.setColor(preColor);
    }

    @Handler
    public void handle(FilterMessage filterMessage) {
        int colorA;
        int colorB;
        int color;

        filteredPixmap = new Pixmap(pixmap.getWidth(), pixmap.getHeight(), pixmap.getFormat());

        switch (filterMessage.getType()) {
            case DFilter:

                long t = System.currentTimeMillis();

                int r1, g1, b1;
                int r2, g2, b2;
                int r, g, b;

                for (int x = 0; x < pixmap.getWidth(); x++) {
                    for (int y = 0; y < pixmap.getHeight(); y++) {
                        colorA = pixmap.getPixel(x, y);
                        colorB = pixmap.getPixel(x + 1, y);

                        r1 = (colorA >> 24) & 0xFF;
                        g1 = (colorA >> 16) & 0xFF;
                        b1 = (colorA >> 8) & 0xFF;

                        r2 = (colorB >> 24) & 0xFF;
                        g2 = (colorB >> 16) & 0xFF;
                        b2 = (colorB >> 8) & 0xFF;

                        r = Math.abs(r2 - r1);
                        g = Math.abs(g2 - g1);
                        b = Math.abs(b2 - b1);

                        color = (r << 24) | (g << 16) | (b << 8) | 0XFF;
                        filteredPixmap.drawPixel(x, y, color);
                    }
                }

                System.out.println(System.currentTimeMillis() - t);

                break;
            case NoFilter:
                filteredPixmap.drawPixmap(pixmap, 0, 0);
                break;
        }

        region = new TextureRegion(new Texture(filteredPixmap));
        setSize(region.getRegionWidth(), region.getRegionHeight());
    }
}
