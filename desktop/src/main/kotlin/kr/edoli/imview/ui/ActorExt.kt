package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.TextTooltip
import kr.edoli.imview.ui.contextmenu.ContextMenu
import kr.edoli.imview.ui.contextmenu.ContextMenuPanel
import kr.edoli.imview.ui.res.uiSkin

fun <T : Actor> T.tooltip(text: String): T {
    addListener(TextTooltip(text, UIFactory.tooltipManager, uiSkin))
    return this
}

fun <T : Actor> T.contextMenu(builder: ContextMenuPanel.() -> Unit): T {
    addListener(ContextMenu(UIFactory.contextMenuManager, builder))
    return this
}