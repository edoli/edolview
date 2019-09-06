package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.ui.TextTooltip
import kr.edoli.imview.ui.contextmenu.ContextMenu
import kr.edoli.imview.ui.contextmenu.ContextMenuPanel

fun <T: Actor> T.tooltip(text: String): T {
    addListener(TextTooltip(text, UIFactory.tooltipManager, UIFactory.skin))
    return this
}
fun <T: Actor> T.contextMenu(builder: ContextMenuPanel.() -> Unit): T {
    addListener(ContextMenu(UIFactory.contextMenuManager).apply {
        builder(rootTable)
    })
    return this
}