package kr.edoli.edoliui.widget

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture

/**
 * Created by daniel on 16. 1. 8.
 */
object Textures {
    val white = produceWhiteTexture()

    private fun produceWhiteTexture(): Texture {
        val pixmap = Pixmap(1, 1, Pixmap.Format.RGB565)

        pixmap.setColor(Color.WHITE)
        pixmap.fill()

        val tex = Texture(pixmap)
        tex.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)

        pixmap.dispose()

        return tex
    }
}
