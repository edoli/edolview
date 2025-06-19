package kr.edoli.edolview.ui.res

import com.badlogic.gdx.graphics.Color
import kotlin.reflect.full.declaredMemberProperties

fun Color.awtColor(): java.awt.Color {
    return java.awt.Color(r, g, b, a)
}

object Colors {
    val transpent = Color.valueOf("00000000")
    val translucent = Color.valueOf("FFFFFF44")

    val background = Color.valueOf("111111")
    val backgroundComponent = Color.valueOf("1F1F1F")
    val backgroundPopup = Color.valueOf("1F1F1F")
    val backgroundOver = Color.valueOf("4D4D4D")
    val backgroundDown = Color.valueOf("313131")
    val backgroundBorder = Color.valueOf("232323")
    val backgroundTextField = Color.valueOf("172B3D")
    val backgroundTooltip = Color.valueOf("000000C0")

    val normal = Color.valueOf("D4D4D4")
    val reverse = Color.valueOf("111111")

    val negative = Color.valueOf("FF6B6B")
    val over = Color.valueOf("FFE66D")
    val inactive = Color.valueOf("8D8D8D")
    val accent = Color.valueOf("4EADE4")
    val accentOver = Color.valueOf("65BEFA")
    val accentDark = Color.valueOf("226699")

    val RED = Color.valueOf("FF0000")
    val GREEN = Color.valueOf("00FF00")
    val BLUE = Color.valueOf("2222FF")
    val GRAY = Color.valueOf("AAAAAA")

    val GRID_STROKE = Color.valueOf("AAAAAA40")
    val VG_TOOLTIP_BG = Color.valueOf("000000C0")
    val VG_TOOLTIP = Color.valueOf("DDDDDD")

    init {
        Colors::class.declaredMemberProperties.forEach {
            com.badlogic.gdx.graphics.Colors.put(it.name, it.get(this) as Color)
        }
    }
}