package kr.edoli.imview.ui;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.scenes.scene2d.ui.Label;
import com.badlogic.gdx.scenes.scene2d.ui.TextButton;
import kr.edoli.imview.ui.drawable.ColorDrawable;

/**
 * Created by 석준 on 2016-02-06.
 */
public class UIFactory {
    private static BitmapFont font = new BitmapFont();

    private static Label.LabelStyle labelStyle = new Label.LabelStyle(font, Color.WHITE);
    private static TextButton.TextButtonStyle textButtonStyle = new TextButton.TextButtonStyle(
            new ColorDrawable(new Color(0, 0, 0, 0.25f)),
            new ColorDrawable(new Color(0.792f, 0.02f, 0.302f, 0.5f)),
            new ColorDrawable(new Color(0, 0, 0.5f, 0.25f)),
            font
            );

    public static Label label(String text) {
        Label label = new Label(text, labelStyle);

        return label;
    }

    public static TextButton textButton(String text) {
        if (textButtonStyle.over == null) {
            textButtonStyle.over = new ColorDrawable(new Color(0, 0, 0, 0.5f));
        }

        TextButton button = new TextButton(text, textButtonStyle);



        return button;
    }
}
