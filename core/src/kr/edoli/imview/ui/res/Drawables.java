package kr.edoli.imview.ui.res;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.g2d.TextureAtlas;
import com.badlogic.gdx.scenes.scene2d.utils.Drawable;
import com.badlogic.gdx.scenes.scene2d.utils.TextureRegionDrawable;

/**
 * Created by daniel on 16. 2. 28.
 */
public class Drawables {
    public static TextureAtlas atlas = new TextureAtlas(Gdx.files.internal("images.atlas"));

    public static Drawable leftArrow = new TextureRegionDrawable(atlas.findRegion("leftArrow"));
    public static Drawable rightArrow = new TextureRegionDrawable(atlas.findRegion("rightArrow"));
}
