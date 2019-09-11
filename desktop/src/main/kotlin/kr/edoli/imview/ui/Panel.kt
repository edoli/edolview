package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Cell
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import kr.edoli.imview.ui.res.uiSkin

open class Panel(showBackground: Boolean = true) : Table(uiSkin) {
    init {
        if (showBackground) {
            background = NinePatchDrawable(skin.atlas.createPatch("default-pane"))
        }
        touchable = Touchable.enabled
    }

    fun addHorizontalDivider(): Cell<ColorRect> {
        row()
        val cell = add(ColorRect(Color.DARK_GRAY)).expandX().fillX().height(1f)
        row()
        return cell
    }

    fun addVerticalDivider(): Cell<ColorRect> {
        return add(ColorRect(Color.DARK_GRAY)).expandY().fillY().width(1f)
    }
}