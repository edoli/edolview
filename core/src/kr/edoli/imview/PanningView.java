package kr.edoli.imview;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.Batch;
import com.badlogic.gdx.graphics.glutils.ShapeRenderer;
import com.badlogic.gdx.math.Vector2;
import com.badlogic.gdx.scenes.scene2d.Actor;
import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.InputListener;
import com.badlogic.gdx.scenes.scene2d.Touchable;
import com.badlogic.gdx.scenes.scene2d.ui.WidgetGroup;
import com.badlogic.gdx.scenes.scene2d.utils.ActorGestureListener;
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils;
import lombok.Getter;

/**
 * Created by 석준 on 2016-02-06.
 */
public class PanningView extends WidgetGroup {

    private Actor actor;
    private @Getter float mouseXOnImage;
    private @Getter float mouseYOnImage;

    private @Getter float mouseXOnPanning;
    private @Getter float mouseYOnPanning;

    private Vector2 mousePosA = new Vector2();

    private ShapeRenderer shapeRenderer = new ShapeRenderer();

    public PanningView(final Actor actor) {
        this.actor = actor;
        addActor(actor);

        setTouchable(Touchable.enabled);

        addListener(new ActorGestureListener() {
            @Override
            public void pan(InputEvent event, float x, float y, float deltaX, float deltaY) {

                if (!UIUtils.ctrl()) {
                    actor.moveBy(deltaX, deltaY);
                }
                super.pan(event, x, y, deltaX, deltaY);
            }
        });


        addListener(new InputListener() {

            @Override
            public boolean scrolled(InputEvent event, float x, float y, int amount) {

                mousePosA.set(x, y);
                actor.parentToLocalCoordinates(mousePosA);

                double factor = Math.pow(1.1, -amount);

                actor.setScale((float) (actor.getScaleX() * factor));


                actor.localToParentCoordinates(mousePosA);

                mousePosA.sub(x, y);
                actor.moveBy(-mousePosA.x, -mousePosA.y);

                return super.scrolled(event, x, y, amount);
            }
        });
    }

    @Override
    public void layout() {
        super.layout();

        actorToCenter();
    }

    private void actorToCenter() {
        actor.setX((getWidth() - actor.getWidth()) / 2);
        actor.setY((getHeight() - actor.getHeight()) / 2);
    }

    public void reset() {
        actorToCenter();
        actor.setScale(1);
    }

    @Override
    public Actor hit(float x, float y, boolean touchable) {
        mousePosA.set(x, y);

        mouseXOnPanning = mousePosA.x;
        mouseYOnPanning = mousePosA.y;

        actor.parentToLocalCoordinates(mousePosA);

        mouseXOnImage = mousePosA.x;
        mouseYOnImage = mousePosA.y;

        mouseXOnImage = Utils.clamp(mouseXOnImage, 0, actor.getWidth());
        mouseYOnImage = Utils.clamp(mouseYOnImage, 0, actor.getHeight());

        return super.hit(x, y, touchable);
    }

    @Override
    public void draw(Batch batch, float parentAlpha) {
        super.draw(batch, parentAlpha);

        batch.end();

        shapeRenderer.begin(ShapeRenderer.ShapeType.Line);
        shapeRenderer.setColor(Color.MAROON);
        shapeRenderer.line(0, mouseYOnPanning + 1, getWidth(), mouseYOnPanning + 1);
        shapeRenderer.line(mouseXOnPanning - 1, 0, mouseXOnPanning - 1, getHeight());
        shapeRenderer.end();

        batch.begin();
    }
}
