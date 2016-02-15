package kr.edoli.imview;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.Pixmap;
import com.badlogic.gdx.graphics.Texture;

/**
 * Created by 석준 on 2016-02-06.
 */
public class Textures {
    public static Texture White = colorTexture(Color.WHITE);

    private static Texture colorTexture(Color color) {
        Pixmap pixmap = new Pixmap(1, 1, Pixmap.Format.RGBA8888);

        pixmap.setColor(color);
        pixmap.fill();

        Texture texture = new Texture(pixmap);

        pixmap.dispose();

        return texture;
    }
}
