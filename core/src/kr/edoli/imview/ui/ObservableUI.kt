package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Button
import com.badlogic.gdx.scenes.scene2d.ui.Label
import kr.edoli.imview.ComparisonMode
import kr.edoli.imview.util.ObservableValue

/**
 * Created by sjjeon on 16. 10. 28.
 */

fun ObservableValue<Boolean>.checkButton(icon: String, size: Int = 24, onClick: (Boolean) -> Unit = {}): Button {
    val button = UI.iconButton(icon, true, size = size)
    button.onClick {
        val newValue = button.isChecked
        update(newValue)
        onClick(newValue)
    }
    subscribe {
        button.isChecked = it
    }
    return button
}

fun ObservableValue<ComparisonMode>.radioButtons(icons: Array<String>, size: Int = 24, onClick: (ComparisonMode) -> Unit = {}): Array<Button> {
    val values = ComparisonMode.values
    val currentValue = get()
    val buttons = Array<Button>(values.size) {
        UI.iconButton(icons[it], true, size = size)
    }

    for ((i, button) in buttons.withIndex()) {
        button.onClick {
            val newValue = values[i]
            buttons.forEach { it.isChecked = false }
            button.isChecked = true
            update(newValue)
            onClick(newValue)
        }
        if (values[i] == currentValue) {
            button.isChecked = true
        }
    }

    return buttons
}

fun <T> ObservableValue<T>.label(format: (T) -> String = {it.toString()}): Label {
    val label = UI.label(format(get()))

    subscribe {
        label.setText(format(it))
    }

    return label
}