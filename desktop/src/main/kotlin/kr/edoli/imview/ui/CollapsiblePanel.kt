package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align

open class CollapsiblePanel(val title: String, val panel: Table) : Panel(false) {
    private val titleLabel = Label(title, UIFactory.skin).apply {
        setAlignment(Align.left)
    }
    private val collapseButton = UIFactory.createIconButton(Ionicons.ionMdFiling) { button ->
        collapse = button.isChecked
    }.apply {
        style = UIFactory.iconToggleButtonStyle
    }

    private val titleTable = Table().apply {
        add(titleLabel).expandX().fillX()
        add(collapseButton)
    }

    var collapse = false
        set(value) {
            clearChildren()
            add(titleTable).expandX().fillX().pad(0f, 4f, 0f, 4f)
            if (!value) {
                row()
                add(panel).expandX().fillX()
            }
            field = value
        }

    init {
        collapse = false
    }


}