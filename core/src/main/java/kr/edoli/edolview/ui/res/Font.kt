package kr.edoli.edolview.ui.res

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.BitmapFont
import com.badlogic.gdx.graphics.g2d.freetype.FreeTypeFontGenerator
import kr.edoli.edolview.util.Platform

object Font {
    val scale = Platform.getScalingFactor()

    fun fontGenerator(
            font: String,
            size: Int,
            action: ((parameter: FreeTypeFontGenerator.FreeTypeFontParameter) -> Unit)? = null
    ): BitmapFont {
        val generator = FreeTypeFontGenerator(Gdx.files.internal(font))
        val parameter = FreeTypeFontGenerator.FreeTypeFontParameter()


        parameter.size = (size * scale).toInt()
        parameter.incremental = true
        if (action != null) {
            action(parameter)
        }
        return generator.generateFont(parameter).apply {
            data.markupEnabled = true
            region.texture.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)
            data.setScale(1.0f / scale)
        }
    }

    val ioniconsFont = fontGenerator("ionicons.ttf", 22)
    val defaultFont = fontGenerator("font.ttf", 16)
}