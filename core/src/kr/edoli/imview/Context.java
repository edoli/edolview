package kr.edoli.imview;

import com.badlogic.gdx.Gdx;
import com.badlogic.gdx.graphics.Pixmap;
import com.badlogic.gdx.math.Rectangle;
import com.badlogic.gdx.math.Vector2;
import com.badlogic.gdx.scenes.scene2d.Stage;
import kr.edoli.imview.ui.PanningView;
import kr.edoli.imview.util.Utils;
import kr.edoli.imview.value.Value;

/**
 * Created by daniel on 16. 2. 27.
 */
public class Context {

    public static Stage stage;
    public static PanningView panningView;

    public static Value<String> imagePath = new Value<>();
    public static Value<Pixmap> currentImage = new Value<>(new Pixmap(0 ,0, Pixmap.Format.RGB888));

    public static Value<Rectangle> selectedRegionOnImage = new Value<>(new Rectangle());

    public static Vector2 getMousePosOnImage() {
        Vector2 tmp = new Vector2();

        if (panningView != null) {
            float x = Gdx.input.getX();
            float y = Gdx.input.getY();

            tmp.set(x, y);
            panningView.screenToLocalCoordinates(tmp);
            panningView.localToImageCoordinates(tmp);

            tmp.x = Utils.clamp(tmp.x, 0, currentImage.get().getWidth());
            tmp.y = Utils.clamp(tmp.y, 0, currentImage.get().getHeight());
        }
        return tmp;
    }

    public static Vector2 getMousePosOnPanning() {
        Vector2 tmp = new Vector2();

        if (panningView != null) {
            float x = Gdx.input.getX();
            float y = Gdx.input.getY();

            tmp.set(x, y);
            panningView.screenToLocalCoordinates(tmp);
        }
        return tmp;

    }
}
