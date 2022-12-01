package kr.edoli.edolview.ui

import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.Cell
import com.badlogic.gdx.scenes.scene2d.ui.Container
import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.drawable.BorderedDrawable
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.util.format
import kotlin.math.pow

class StatusBar : Panel() {
    companion object {
        const val rowHeight = 24f
        const val barHeight = rowHeight * 2
    }

    init {
        background = BorderedDrawable(Colors.background, Colors.backgroundBorder).apply {
            bottomBorder = false
            leftBorder = false
            rightBorder = false
        }

        align(Align.left and Align.center)
        add(UIFactory.createColorRect(ImContext.marqueeBoxRGB).tooltip("Marquee RGB")).width(rowHeight).height(rowHeight)
        addContainer(UIFactory.createColorLabel(ImContext.marqueeBoxRGB).tooltip("Marquee RGB")).width(600f).height(rowHeight)
        addContainer(UIFactory.createRectField(ImContext.marqueeBox).tooltip("Marquee bound"), true).width(196f).height(rowHeight)
        add().expandX().fillX()

        row()
        add(UIFactory.createColorRect(ImContext.cursorRGB).tooltip("Cursors RGB")).width(rowHeight).height(rowHeight)
        addContainer(UIFactory.createColorLabel(ImContext.cursorRGB).tooltip("Cursors RGB")).width(600f).height(rowHeight)
        addContainer(UIFactory.createPointLabel(ImContext.cursorPosition).tooltip("Cursor position")).width(196f).height(rowHeight)
        add().expandX().fillX()
        add(UIFactory.createLabel(ImContext.zoom) { "${(it * 100).format(2)}%" }).height(rowHeight)
    }

    fun addContainer(actor: Actor, fill: Boolean=false): Cell<Actor> {
        return add(Container(actor).apply {
            if (fill) {
                fillX()
            }
            align(Align.left)
        })
    }
}