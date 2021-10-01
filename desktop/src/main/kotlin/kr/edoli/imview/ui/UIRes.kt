package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture

object UIRes {

    val white = produceWhiteTexture()

    private fun produceWhiteTexture(): Texture {
        val pixmap = Pixmap(1, 1, Pixmap.Format.RGB888)

        pixmap.setColor(Color.WHITE)
        pixmap.fill()

        val tex = Texture(pixmap)
        tex.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)
        tex.setWrap(Texture.TextureWrap.Repeat, Texture.TextureWrap.Repeat)

        pixmap.dispose()

        return tex
    }
}

