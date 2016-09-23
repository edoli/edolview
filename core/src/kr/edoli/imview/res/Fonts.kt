package kr.edoli.edoliui.res

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.g2d.BitmapFont
import com.badlogic.gdx.graphics.g2d.freetype.FreeTypeFontGenerator

/**
 * Created by daniel on 16. 6. 4.
 */
object Fonts {

    val iconicFont: BitmapFont = iconic()
    val textFont: BitmapFont = text()

    private fun iconic() : BitmapFont {
        val generator = FreeTypeFontGenerator(Gdx.files.internal("fontawesome.ttf"))
        val parameter = FreeTypeFontGenerator.FreeTypeFontParameter()
        parameter.size = 16
        parameter.incremental = true
        return generator.generateFont(parameter)
    }

    private fun text() : BitmapFont {
        val generator = FreeTypeFontGenerator(Gdx.files.internal("font.ttf"))
        val parameter = FreeTypeFontGenerator.FreeTypeFontParameter()
        parameter.size = 16
        parameter.hinting = FreeTypeFontGenerator.Hinting.Full
        parameter.incremental = true
        return generator.generateFont(parameter)
    }
}