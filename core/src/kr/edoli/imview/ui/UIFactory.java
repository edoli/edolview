package kr.edoli.imview.ui;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.graphics.g2d.BitmapFont;
import com.badlogic.gdx.scenes.scene2d.ui.*;
import com.badlogic.gdx.scenes.scene2d.utils.Drawable;
import kr.edoli.imview.ui.drawable.ColorBorderDrawable;
import kr.edoli.imview.ui.drawable.ColorDrawable;
import kr.edoli.imview.ui.res.Colors;

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

    private static ImageButton.ImageButtonStyle imageButtonStyle;
    private static TextButton.TextButtonStyle checkButtonStyle;
    private static CheckBox.CheckBoxStyle checkBoxStyle;

    public static Label label(String text) {
        Label label = new Label(text, labelStyle);

        return label;
    }

    public static ImageButton iconButton(Drawable icon) {
        if (imageButtonStyle == null) {
            imageButtonStyle = new ImageButton.ImageButtonStyle();

            imageButtonStyle.up = new ColorBorderDrawable(Colors.background, Colors.border);
            imageButtonStyle.over = new ColorBorderDrawable(Colors.backgroundOver, Colors.border);
        }
        if (icon != null) {
            imageButtonStyle.imageUp = icon;
        }
        ImageButton button = new ImageButton(imageButtonStyle);

        return button;

    }

    public static TextButton textButton(String text) {
        if (textButtonStyle.over == null) {
            textButtonStyle.over = new ColorDrawable(new Color(0, 0, 0, 0.5f));
        }

        TextButton button = new TextButton(text, textButtonStyle);

        return button;
    }

    public static Button checkBox(String text) {
        if (checkButtonStyle == null) {
            checkButtonStyle = new TextButton.TextButtonStyle();

            checkButtonStyle.font = font;

            checkButtonStyle.up = new ColorBorderDrawable(Colors.background, Colors.border);
            checkButtonStyle.over = new ColorBorderDrawable(Colors.backgroundOver, Colors.border);

            checkButtonStyle.checked = new ColorBorderDrawable(Colors.checked, Colors.border);
            checkButtonStyle.checkedOver = new ColorBorderDrawable(Colors.checkedOver, Colors.border);
        }

        TextButton button = new TextButton(text, checkButtonStyle);

        return button;
    }
}
