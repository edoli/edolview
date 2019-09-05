package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext

class StatusBar : Table() {
    companion object {
        const val barHeight = 24f
    }

    init {
        background = NinePatchDrawable(UIFactory.skin.atlas.createPatch("default-pane"))

        align(Align.left and Align.center)
        add(UIFactory.createColorRect(ImContext.cursorRGB)).width(barHeight).height(barHeight)
        add(UIFactory.createColorLabel(ImContext.cursorRGB)).width(196f).height(barHeight)
        add(UIFactory.createColorRect(ImContext.marqueeBoxRGB)).width(barHeight).height(barHeight)
        add(UIFactory.createColorLabel(ImContext.marqueeBoxRGB)).width(196f).height(barHeight)
        add(UIFactory.createRectLabel(ImContext.marqueeBox)).width(196f).height(barHeight)
        add(UIFactory.createPointLabel(ImContext.cursorPosition)).width(196f).height(barHeight)
        add().expandX().fillX()

    }
}