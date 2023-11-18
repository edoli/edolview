package kr.edoli.edolview.ui.res

import com.badlogic.gdx.graphics.Color
import kotlin.reflect.full.declaredMemberProperties

fun Color.awtColor(): java.awt.Color {
    return java.awt.Color(r, g, b, a)
}

object Colors {
    val transpent = Color.valueOf("00000000")

    val background = Color.valueOf("474747")
    val backgroundDark = Color.valueOf("363636")
    val backgroundContextMenu = Color.valueOf("272727")
    val backgroundOver = Color.valueOf("676767")
    val backgroundDown = Color.valueOf("272727")
    val backgroundBorder = Color.valueOf("333333")

    val normal = Color.valueOf("F7FFF7")
    val negative = Color.valueOf("FF6B6B")
    val over = Color.valueOf("FFE66D")
    val inactive = Color.valueOf("ACACAC")
    val accent = Color.valueOf("4ECDC4")
    val accentDark = Color.valueOf("2E8D84")
    val accentSemi = Color.valueOf("4ECDC488")
    val accentDarkSemi = Color.valueOf("2E8D8488")

    init {
        Colors::class.declaredMemberProperties.forEach {
            com.badlogic.gdx.graphics.Colors.put(it.name, it.get(this) as Color)
        }
    }
}