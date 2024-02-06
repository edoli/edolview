package kr.edoli.edolview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Cursor
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.ui.TextTooltip
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import kr.edoli.edolview.ui.contextmenu.ContextMenu
import kr.edoli.edolview.ui.contextmenu.ContextMenuPanel
import kr.edoli.edolview.ui.res.uiSkin

fun <T : Actor> T.tooltip(text: String?): T {
    if (text != null) {
        addListener(TextTooltip(text, UIFactory.tooltipManager, uiSkin))
    }
    return this
}

fun <T : Actor> T.contextMenu(builder: ContextMenuPanel.() -> Unit): T {
    val contextMenuListeners = listeners.filterIsInstance<ContextMenu>()
    if (contextMenuListeners.isEmpty()) {
        addListener(ContextMenu(UIFactory.contextMenuManager, builder))
    } else {
        contextMenuListeners[0].builders.add(builder)
    }
    return this
}

fun <T: Actor> T.cursor(cursor: Cursor.SystemCursor): T {
    addListener(object : InputListener() {
        override fun enter(event: InputEvent?, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
            Gdx.graphics.setSystemCursor(cursor)
        }
    })
    return this
}

var presentationFocus: Actor? = null

fun <T: Actor> T.presentation(): T {
    addListener(object : InputListener() {
        override fun enter(event: InputEvent, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
            presentationFocus = this@presentation
        }

        override fun exit(event: InputEvent, x: Float, y: Float, pointer: Int, toActor: Actor?) {
            if (presentationFocus == this@presentation) {
                presentationFocus = null
            }
        }
    })
    return this
}