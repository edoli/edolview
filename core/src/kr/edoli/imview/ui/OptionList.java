package kr.edoli.imview.ui;

import com.badlogic.gdx.scenes.scene2d.InputEvent;
import com.badlogic.gdx.scenes.scene2d.ui.Button;
import com.badlogic.gdx.scenes.scene2d.ui.ButtonGroup;
import com.badlogic.gdx.scenes.scene2d.ui.Table;
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener;
import com.badlogic.gdx.utils.Array;
import kr.edoli.imview.ui.util.UIFactory;

/**
 * Created by sjjeon on 16. 4. 21.
 */
public class OptionList extends Table {

    private int currentId = 0;
    private OptionChangedListener optionChangedListener;
    private Array<Button> buttons = new Array<>();
    private Button currentCheckedButton;
    private int currentCheckedButtonId;
    private ButtonGroup<Button> optionGroup = new ButtonGroup<>();

    public OptionList() {
    }

    public OptionList(String[] options) {
        for (String option : options) {
            addOption(option, false);
        }
        refresh();
    }

    private void addOption(String text, boolean refresh) {
        final Button option = UIFactory.checkBox(text);
        final int id = currentId++;
        option.pad(4, 8, 4, 8);
        optionGroup.add(option);

        buttons.add(option);

        option.addListener(new ClickListener() {
            @Override
            public void clicked(InputEvent event, float x, float y) {
                if (option.isChecked()) {
                    changed(option, id);
                }
            }
        });

    }

    public void addOption(String text) {
        addOption(text, true);
    }

    public void refresh() {
        clearChildren();

        if (currentCheckedButton == null) {
            currentCheckedButton = buttons.get(0);
            changed(currentCheckedButton, 0);
        }

        for (Button button : buttons) {
            add(button).expandX().fillX();
        }
    }

    private void changed(Button option, int id) {
        if (optionChangedListener != null) {
            optionChangedListener.changed(option, id);
        }
        currentCheckedButton = option;
        currentCheckedButtonId = id;
    }

    public void setOptionChangedListener(OptionChangedListener listener) {
        this.optionChangedListener = listener;
        listener.changed(currentCheckedButton, currentCheckedButtonId);
    }


    public interface OptionChangedListener {
        void changed(Button option, int id);
    }
}
