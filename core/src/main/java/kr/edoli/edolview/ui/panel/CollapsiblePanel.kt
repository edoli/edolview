package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.ui.res.Ionicons

open class CollapsiblePanel(val title: String, val panel: Table, private val initCollapse: Boolean = false)
    : Panel(false) {

    var onCollapseChanged: (Boolean) -> Unit = {}

    private val titleLabel = Label(title, skin).apply {
        setAlignment(Align.left)
    }

    private val collapseButton = UIFactory.createIconButton(Ionicons.ionMdFiling) { button ->
        collapse = button.isChecked
    }.apply {
        style = UIFactory.iconToggleButtonStyle
        isChecked = initCollapse
    }

    private val titleTable = Table().apply {
        add(titleLabel).expandX().fillX()
        add(collapseButton)
    }

    var collapse = initCollapse
        set(value) {
            clearChildren()
            if (panel is Panel) {
                panel.onGoneChanged(value)
            }
            add(titleTable).expandX().fillX().pad(0f, 4f, 0f, 4f)
            if (!value) {
                row()
                add(panel).expandX().fillX()
            }
            onCollapseChanged(value)
            field = value
        }

    init {
        collapse = initCollapse
    }
}