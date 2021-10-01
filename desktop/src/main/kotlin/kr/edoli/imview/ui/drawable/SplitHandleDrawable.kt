package kr.edoli.imview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import kr.edoli.imview.ui.UIRes


class SplitHandleDrawable(fill: Color, border: Color, val dot: Color,
                          private val size: Float,
                          private val isVertical: Boolean = false)
    : BorderedDrawable(fill, border) {

    init {
        if (isVertical) {
            leftBorder = false
            rightBorder = false
            minHeight = size
        } else {
            topBorder = false
            bottomBorder = false
            minWidth = size
        }
    }

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        super.draw(batch, x, y, width, height)

        val centerX = x + width / 2
        val centerY = y + height / 2

        val dotSize = size - borderWidth * 4
        val offset = dotSize * 4

        val dotX = centerX - dotSize / 2
        val dotY = centerY - dotSize / 2
        batch.color = dot
        batch.draw(UIRes.white, dotX, dotY, dotSize, dotSize)

        if (isVertical) {
            batch.draw(UIRes.white, dotX - offset, dotY, dotSize, dotSize)
            batch.draw(UIRes.white, dotX + offset, dotY, dotSize, dotSize)
        } else {
            batch.draw(UIRes.white, dotX, dotY - offset, dotSize, dotSize)
            batch.draw(UIRes.white, dotX, dotY + offset, dotSize, dotSize)
        }
    }
}