package kr.edoli.edolview.ui

import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Cell
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ui.drawable.BorderedDrawable
import kr.edoli.edolview.ui.panel.CollapsiblePanel
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.ui.res.uiSkin

open class Panel(showBackground: Boolean = true) : Table(uiSkin) {

    var onGoneChanged: (isGone: Boolean) -> Unit = {}

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

    fun isGone(): Boolean {
        if (stage == null) {
            return true
        }
        val parent = this.parent
        if (parent is CollapsiblePanel) {
            return parent.isGone()
        }
        return false
    }
}