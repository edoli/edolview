package kr.edoli.edolview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.edolview.ui.drawLine
import kr.edoli.edolview.ui.drawPolygon
import kr.edoli.edolview.ui.drawRectBorder

class CursorDrawable(
    val color: Color,
    val thickness: Float = 1f
) : BaseDrawable() {

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = color
        val xCenter = x + width / 2
        batch.drawLine(xCenter, y, xCenter, y + height, thickness)

        super.draw(batch, x, y, width, height)
    }

    override fun getMinWidth(): Float {
        return 3f
    }

    fun pad(value: Float): CursorDrawable {
        setPadding(value, value, value, value)
        return this
    }

    fun pad(top: Float, left: Float, bottom: Float, right: Float): CursorDrawable {
        setPadding(top, left, bottom, right)
        return this
    }
}