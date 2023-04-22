package kr.edoli.edolview.ui.custom

import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.BitmapFont
import com.badlogic.gdx.graphics.g2d.BitmapFont.BitmapFontData
import com.badlogic.gdx.graphics.g2d.GlyphLayout
import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.ui.Skin
import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.util.clamp

class CustomList<T>(style: ListStyle, val textFunc: (T) -> String) : List<T>(style) {

    constructor(skin: Skin, textFunc: (T) -> String) : this(skin[ListStyle::class.java], textFunc)
    override fun toString(item: T): String {
        return textFunc(item)
    }

    override fun drawItem(
        batch: Batch,
        font: BitmapFont,
        index: Int,
        item: T,
        x: Float,
        y: Float,
        width: Float
    ): GlyphLayout {
        val string = toString(item)

        if (alignment and Align.right != 0) {
            val fontData = font.data

            var totalWidth = 0f
            var strIndex = string.length - 1

            totalWidth += getGlyphWidth(fontData.getGlyph('.'), fontData) * 3

            while (totalWidth < width - 0.0001f && strIndex >= 0) {

                val glyph = fontData.getGlyph(string[strIndex])
                totalWidth += getGlyphWidth(glyph, fontData) * 1.1f

                strIndex -= 1
            }
            val start = (strIndex + 2).clamp(0, string.length - 1)
            val newStr = "..." + string.subSequence(start, string.length)
            return font.draw(batch, newStr, x, y, 0, newStr.length, width, alignment, false, null)
        } else {
            return font.draw(batch, string, x, y, 0, string.length, width, alignment, false, "...")
        }
    }

    /** Returns the distance from the glyph's drawing position to the right edge of the glyph.  */
    private fun getGlyphWidth(glyph: BitmapFont.Glyph, fontData: BitmapFontData): Float {
        return if (glyph.fixedWidth) {
            glyph.xadvance * scaleX
        } else {
            (glyph.width + glyph.xoffset) * fontData.scaleX - fontData.padRight
        }
    }
}