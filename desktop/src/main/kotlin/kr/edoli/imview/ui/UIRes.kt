package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable

object UIRes {

    val white = produceWhiteTexture()

    private fun produceWhiteTexture(): Texture {
        val pixmap = Pixmap(1, 1, Pixmap.Format.RGB888)

        pixmap.setColor(Color.WHITE)
        pixmap.fill()

        val tex = Texture(pixmap)
        tex.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)

        pixmap.dispose()

        return tex
    }
}

class ColorDrawable(val color: Color) : BaseDrawable() {

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = color
        batch.draw(UIRes.white, x, y, width, height)
    }
}

class BorderedDrawable(val fill: Color, val border: Color) : BaseDrawable() {

    var borderWidth = 1f
    var isBorder = true
    var topBorder = true
    var bottomBorder = true
    var leftBorder = true
    var rightBorder = true

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = fill
        batch.draw(UIRes.white, x, y, width, height)

        if (isBorder) {
            batch.color = border
            if (topBorder) {
                batch.draw(UIRes.white, x, y, width, borderWidth)
            }
            if (leftBorder) {
                batch.draw(UIRes.white, x, y, borderWidth, height)
            }
            if (bottomBorder) {
                batch.draw(UIRes.white, x, y + height - borderWidth, width, borderWidth)
            }
            if (rightBorder) {
                batch.draw(UIRes.white, x + width - borderWidth, y, borderWidth, height)
            }
        }
    }
}