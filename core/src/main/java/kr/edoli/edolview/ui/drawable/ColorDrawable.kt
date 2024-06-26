package kr.edoli.edolview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.edolview.ui.UIRes


class ColorDrawable(val color: Color) : BaseDrawable() {

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = color
        batch.draw(UIRes.white, x, y, width, height)
    }

    fun pad(all: Float) {
        topHeight = all
        rightWidth = all
        bottomHeight = all
        leftWidth = all
    }

    fun pad(top: Float, right: Float, bottom: Float, left: Float) {
        topHeight = top
        rightWidth = right
        bottomHeight = bottom
        leftWidth = left
    }
}