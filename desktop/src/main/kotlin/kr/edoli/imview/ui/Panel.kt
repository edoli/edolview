package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Cell
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ui.drawable.BorderedDrawable
import kr.edoli.imview.ui.panel.CollapsiblePanel
import kr.edoli.imview.ui.res.Colors
import kr.edoli.imview.ui.res.uiSkin

open class Panel(showBackground: Boolean = true) : Table(uiSkin) {

    var onGoneChanged: (isGone: Boolean) -> Unit = {}

    init {
        if (showBackground) {
            background = BorderedDrawable(Colors.background, Colors.backgroundBorder)
        }
        touchable = Touchable.enabled
    }

    fun addHorizontalDivider(thickness: Float = 1f): Cell<ColorRect> {
        row()
        val cell = add(ColorRect(Colors.backgroundBorder)).expandX().fillX().height(thickness)
        row()
        return cell
    }

    fun addVerticalDivider(thickness: Float = 1f): Cell<ColorRect> {
        return add(ColorRect(Colors.backgroundBorder)).expandY().fillY().width(thickness)
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