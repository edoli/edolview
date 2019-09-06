package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Cell
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable

open class Panel(showBackground: Boolean = true) : Table(UIFactory.skin) {
    init {
        if (showBackground) {
            background = NinePatchDrawable(UIFactory.skin.atlas.createPatch("default-pane"))
        }
        touchable = Touchable.enabled
    }

    fun addHorizontalDivider(): Cell<ColorRect> {
        return add(ColorRect(Color.DARK_GRAY)).expandX().fillX().height(1f)
    }

    fun addVerticalDivider(): Cell<ColorRect> {
        return add(ColorRect(Color.DARK_GRAY)).expandY().fillY().width(1f)
    }
}