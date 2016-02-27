package kr.edoli.imview.ui;

import com.badlogic.gdx.graphics.Color;
import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.ui.ButtonGroup;
import com.badlogic.gdx.scenes.scene2d.ui.Table;
import com.badlogic.gdx.scenes.scene2d.ui.TextButton;
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener;
import com.badlogic.gdx.utils.Align;
import kr.edoli.imview.Bus;
import kr.edoli.imview.event.FilterMessage;
import kr.edoli.imview.ui.drawable.ColorDrawable;

/**
 * Created by 석준 on 2016-02-06.
 */
public class FilterView extends Table {

    private ButtonGroup buttonGroup = new ButtonGroup();

    public FilterView() {
        align(Align.top);
        background(new ColorDrawable(new Color(0, 0, 0, 0.5f)));

        addFilterButton("No Filter", FilterMessage.FilterType.NoFilter);
        addFilterButton("1D Filter", FilterMessage.FilterType.DFilter);
    }

    private void addFilterButton(String text, final FilterMessage.FilterType type) {

        TextButton button = UIFactory.textButton(text);
        button.addListener(new ClickListener() {
            @Override
            public void clicked(InputEvent event, float x, float y) {
                Bus.publish(new FilterMessage(type));
                super.clicked(event, x, y);
            }
        });

        add(button).expandX().fillX().height(32).row();

        buttonGroup.add(button);
    }
}
