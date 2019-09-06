package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.Drawable


class ColorDrawable(val color: Color) : Drawable {
    override fun setRightWidth(rightWidth: Float) {}

    override fun getLeftWidth() = 0f

    override fun setMinHeight(minHeight: Float) {}

    override fun setBottomHeight(bottomHeight: Float) {}

    override fun setTopHeight(topHeight: Float) {}

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        val preColor = batch.color
        batch.color = color
        batch.draw(ColorRect.white, x, y, width, height)
        batch.color = preColor
    }

    override fun getBottomHeight() = 0f

    override fun getRightWidth() = 0f

    override fun getMinWidth() = 0f

    override fun getTopHeight() = 0f

    override fun setMinWidth(minWidth: Float) {}

    override fun setLeftWidth(leftWidth: Float) {}

    override fun getMinHeight() = 0f

}
