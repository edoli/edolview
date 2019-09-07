package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.Actor

class ColorRect(initColor: Color? = null) : Actor() {
    init {
        if (initColor != null) {
            this.color.set(initColor)
        }
    }

    companion object {
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

    override fun draw(batch: Batch, parentAlpha: Float) {
        batch.color = color
        batch.draw(white, x, y, width, height)
    }
}