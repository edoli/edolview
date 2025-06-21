package kr.edoli.edolview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.edolview.ui.UIRes
import kr.edoli.edolview.ui.drawLine
import kotlin.math.min


class WindowDrawable(val backgroundColor: Color, val titlebarColor: Color, val borderColor: Color) : BaseDrawable() {

    init {
        topHeight = 24f
        minHeight = 24f

        leftWidth = 8f
        rightWidth = 8f
        bottomHeight = 8f
    }

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = backgroundColor
        batch.draw(UIRes.white, x, y, width, height)

        val titleBarHeight = min(height, topHeight)
        batch.color = titlebarColor
        batch.draw(UIRes.white, x, y + height - titleBarHeight, width, titleBarHeight)

        batch.color = borderColor
        batch.drawLine(x, y, x + width, y, 1f)
        batch.drawLine(x, y + height, x + width, y + height, 1f)
        batch.drawLine(x, y, x, y + height, 1f)
        batch.drawLine(x + width, y, x + width, y + height, 1f)
    }
}