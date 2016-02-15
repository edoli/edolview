package kr.edoli.imview;

import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.ui.Widget;
import com.badlogic.gdx.scenes.scene2d.utils.ActorGestureListener;

/**
 * Created by 석준 on 2016-02-06.
 */
public class Header extends Widget {
    public Header() {

        addListener(new ActorGestureListener() {
            @Override
            public void pan(InputEvent event, float x, float y, float deltaX, float deltaY) {

                super.pan(event, x, y, deltaX, deltaY);
            }
        });
    }
}
