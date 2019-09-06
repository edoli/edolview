package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.g2d.BitmapFont
import com.badlogic.gdx.graphics.g2d.freetype.FreeTypeFontGenerator

object Font {
    fun fontGenerator(
            font: String,
            size: Int,
            action: ((parameter: FreeTypeFontGenerator.FreeTypeFontParameter) -> Unit)? = null
    ): BitmapFont {
        val generator = FreeTypeFontGenerator(Gdx.files.internal(font))
        val parameter = FreeTypeFontGenerator.FreeTypeFontParameter()
        parameter.size = size
        parameter.incremental = true
        if (action != null) {
            action(parameter)
        }
        return generator.generateFont(parameter).apply {
            data.markupEnabled = true
        }
    }

    val ioniconsFont = fontGenerator("ionicons.ttf", 22)
}