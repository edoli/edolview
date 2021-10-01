package kr.edoli.imview.ui.contextmenu

import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.ui.TextButton
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.drawable.ColorDrawable
import kr.edoli.imview.ui.res.Font
import kotlin.math.max
import kotlin.math.min

class ContextMenu(private val contextMenuManager: ContextMenuManager, builder: (ContextMenuPanel.() -> Unit)?) : ClickListener(Input.Buttons.RIGHT) {
    val rootTable = ContextMenuPanel()
    val location = Vector2()
    val builders = if (builder != null) arrayListOf(builder) else arrayListOf()

    override fun clicked(event: InputEvent, x: Float, y: Float) {
        rootTable.clearChildren()
        builders.forEach { it(rootTable) }
        contextMenuManager.openMenu(event.target, this)

        location.set(x, y)
        event.target.localToStageCoordinates(location)

        rootTable.x = min(location.x, event.target.stage.width - 7f - rootTable.width)
        rootTable.y = max(location.y - rootTable.height, 7f)
    }
}

class ContextMenuPanel : Panel() {
    init {
        align(Align.left)
    }

    val menuButtonStyle = TextButton.TextButtonStyle(
            ColorDrawable(Color(0f, 0f, 0f, 0f)),
            ColorDrawable(Color(1f, 1f, 1f, 0.2f)),
            null,
            Font.defaultFont
    ).apply {
        over = ColorDrawable(Color(0f, 0f, 0f, 0.2f))
    }

    fun addMenu(text: String, action: () -> Unit) {
        add(TextButton(text, menuButtonStyle).apply {
            label.setAlignment(Align.left)
            pad(2f, 4f, 2f, 16f)

            addListener(object : ClickListener() {
                override fun clicked(event: InputEvent?, x: Float, y: Float) {
                    action()
                }
            })
        }).fillX()
        row()
    }

    fun addMenuDivider() {
        addHorizontalDivider().pad(3f, 0f, 3f, 0f)
    }
}