package kr.edoli.edolview.ui.custom

import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.BitmapFont
import com.badlogic.gdx.graphics.g2d.BitmapFont.BitmapFontData
import com.badlogic.gdx.graphics.g2d.GlyphLayout
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.ui.Skin
import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.image.ClipboardUtils
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.ui.contextmenu.ContextMenu
import kr.edoli.edolview.ui.contextmenu.ContextMenuPanel
import kr.edoli.edolview.util.clamp
import javax.naming.Context
import kotlin.math.max
import kotlin.math.min

class CustomList<T>(style: ListStyle, val textFunc: (T) -> String) : List<T>(style) {

    constructor(skin: Skin, textFunc: (T) -> String) : this(skin[ListStyle::class.java], textFunc)

    init {
        addListener(object : InputListener() {
            var pressedIndex = -1

            val contextMenu = ContextMenu(UIFactory.contextMenuManager) {
                addMenu("Copy") {
                    if (pressedIndex != -1) {
                        val item = items[pressedIndex]
                        ClipboardUtils.putString(toString(item))
                    }
                }
            }

            override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                if (pointer != 0 || button != 1) return true
                if (items.size == 0) return true
                val index = getItemIndexAt(y)
                if (index == -1) return true
                pressedIndex = index
                return true
            }

            override fun touchUp(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int) {
                if (pointer != 0 || button != 1) return
                if (pressedIndex != -1) {
                    contextMenu.clicked(event, x, y)
                }
            }
        })
    }
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
            val start = (strIndex + 1).clamp(0, string.length - 1)
            val newStr = if (start < 3) string else "..." + string.subSequence(start, string.length)
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