package kr.edoli.edolview.ui.res

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.edolview.ui.drawPolygon
import kr.edoli.edolview.ui.drawRectBorder

class SelectBoxDrawable : BaseDrawable() {
    val borderColor = Color(0x202020ff)

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = borderColor
        batch.drawRectBorder(x, y, width, height, 1f)
        batch.color = Color.LIGHT_GRAY
        val tx = x + width - 14f
        val ty = y + height / 2 + 4f
        val tw = 10f
        val th = 8f
        (batch as PolygonSpriteBatch).drawPolygon(floatArrayOf(tx, ty, tx + tw, ty, tx + tw / 2, ty - th))
        super.draw(batch, x, y, width, height)
    }

    fun pad(value: Float): SelectBoxDrawable {
        setPadding(value, value, value, value)
        return this
    }

    fun pad(top: Float, left: Float, bottom: Float, right: Float): SelectBoxDrawable {
        setPadding(top, left, bottom, right)
        return this
    }
}