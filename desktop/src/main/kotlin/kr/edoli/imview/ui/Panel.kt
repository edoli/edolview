package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Cell
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ui.res.Colors
import kr.edoli.imview.ui.res.uiSkin

open class Panel(showBackground: Boolean = true) : Table(uiSkin) {
    init {
        if (showBackground) {
            background = BorderedDrawable(Colors.background, Colors.backgroundBorder)
        }
        touchable = Touchable.enabled
    }

    fun addHorizontalDivider(): Cell<ColorRect> {
        row()
        val cell = add(ColorRect(Colors.backgroundBorder)).expandX().fillX().height(1f)
        row()
        return cell
    }

    fun addVerticalDivider(): Cell<ColorRect> {
        return add(ColorRect(Colors.backgroundBorder)).expandY().fillY().width(1f)
    }
}