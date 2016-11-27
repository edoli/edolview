package kr.edoli.edoliui.res

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.BitmapFont
import com.badlogic.gdx.graphics.g2d.freetype.FreeTypeFontGenerator
import java.util.*

/**
 * Created by daniel on 16. 6. 4.
 */
object Fonts {

    private val iconFontMap = HashMap<Int, BitmapFont>()
    private val textFontMap = HashMap<Int, BitmapFont>()

    fun icon(size: Int) : BitmapFont {
        if (iconFontMap.containsKey(size)) {
            return iconFontMap[size] as BitmapFont
        }

        val generator = FreeTypeFontGenerator(Gdx.files.internal("fontawesome.ttf"))
        val parameter = FreeTypeFontGenerator.FreeTypeFontParameter()
        parameter.size = size
        parameter.incremental = true

        val font = generator.generateFont(parameter)
        font.region.texture.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)

        iconFontMap.put(size, font)

        return font
    }

    fun text(size: Int) : BitmapFont {
        if (textFontMap.containsKey(size)) {
            return textFontMap[size] as BitmapFont
        }

        val generator = FreeTypeFontGenerator(Gdx.files.internal("font.ttf"))
        val parameter = FreeTypeFontGenerator.FreeTypeFontParameter()
        parameter.size = size
        parameter.incremental = true

        val font = generator.generateFont(parameter)
        font.region.texture.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)

        textFontMap.put(size, font)

        return font
    }
}