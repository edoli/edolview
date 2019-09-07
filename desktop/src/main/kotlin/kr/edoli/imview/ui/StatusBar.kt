package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.Cell
import com.badlogic.gdx.scenes.scene2d.ui.Container
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext
import kr.edoli.imview.util.format
import java.lang.Math.pow
import kotlin.math.pow

class StatusBar : Panel() {
    companion object {
        const val barHeight = 24f
    }

    init {
        background = NinePatchDrawable(UIFactory.skin.atlas.createPatch("default-pane"))

        align(Align.left and Align.center)
        add(UIFactory.createColorRect(ImContext.cursorRGB).tooltip("Cursors RGB")).width(barHeight).height(barHeight)
        addContainer(UIFactory.createColorLabel(ImContext.cursorRGB).tooltip("Cursors RGB")).width(196f).height(barHeight)
        add(UIFactory.createColorRect(ImContext.marqueeBoxRGB).tooltip("Marquee RGB")).width(barHeight).height(barHeight)
        addContainer(UIFactory.createColorLabel(ImContext.marqueeBoxRGB).tooltip("Marquee RGB")).width(196f).height(barHeight)
        addContainer(UIFactory.createRectLabel(ImContext.marqueeBox).tooltip("Marquee bound")).width(196f).height(barHeight)
        addContainer(UIFactory.createPointLabel(ImContext.cursorPosition).tooltip("Cursor position")).width(196f).height(barHeight)
        add().expandX().fillX()
        add(UIFactory.createLabel(ImContext.zoomLevel) { "${(1.1.pow(it.toDouble()) * 100).format(2)}%" }).height(barHeight)
    }

    fun addContainer(actor: Actor): Cell<Actor> {
        return add(Container(actor).apply {
            align(Align.left)
        })
    }
}