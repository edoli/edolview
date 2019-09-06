package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable

class ToolBar : Table() {

    companion object {
        const val barHeight = 24f
    }

    init {
        background = NinePatchDrawable(UIFactory.skin.atlas.createPatch("default-pane"))
    }
}