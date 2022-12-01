package kr.edoli.edolview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.ui.Widget

class ColorRect(initColor: Color? = null) : Widget() {
    init {
        if (initColor != null) {
            this.color.set(initColor)
        }
    }

    override fun draw(batch: Batch, parentAlpha: Float) {
        batch.color = color
        batch.draw(UIRes.white, x, y, width, height)
    }
}