package kr.edoli.imview.ui.view

import com.badlogic.gdx.scenes.scene2d.Group
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Touchable

/**
 * Created by daniel on 16. 9. 23.
 */
class ContextGroup : Group() {
    val contextMenu = ContextMenu(this)

    init {
        addActor(contextMenu)
        addListener(object : InputListener() {
            override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                if (event.target != contextMenu) {
                    hide()
                }
                return super.touchDown(event, x, y, pointer, button)
            }
        })
        hide()
    }

    fun menuPosition(x: Float, y: Float) {
        contextMenu.x = if (x + contextMenu.width < width) x else x - contextMenu.width
        contextMenu.y = if (y - contextMenu.height < 0) y else y - contextMenu.height
    }

    fun show() {
        touchable = Touchable.enabled
        contextMenu.isVisible = true
    }

    fun hide() {
        touchable = Touchable.disabled
        contextMenu.isVisible = false
    }
}