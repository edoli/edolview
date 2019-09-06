package kr.edoli.imview.ui.contextmenu

import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Stage

class ContextMenuManager {

    var showingContextMenu: ContextMenu? = null
    var stage: Stage? = null

    fun updateStage(stage: Stage) {
        if (this.stage == null) {
            this.stage = stage.apply {
                addListener(object : InputListener() {
                    override fun touchDown(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                        hideMenu()
                        return super.touchDown(event, x, y, pointer, button)
                    }
                })
            }
        }
    }

    fun openMenu(owner: Actor, contextMenu: ContextMenu) {
        updateStage(owner.stage)

        owner.stage.addActor(contextMenu.rootTable)
        showingContextMenu = contextMenu
        contextMenu.rootTable.let {
            it.isVisible = true
            it.pack()
        }
    }

    fun hideMenu() {
        showingContextMenu?.let { contextMenu ->
            contextMenu.rootTable.remove()
        }
    }
}