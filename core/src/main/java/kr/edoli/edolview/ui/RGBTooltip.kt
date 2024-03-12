package kr.edoli.edolview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.res.uiSkin
import kr.edoli.edolview.util.toColor

class RGBTooltip: Table() {
    companion object {
        const val rowHeight = 24f
    }

    init {
        background = uiSkin.getDrawable("tooltip_background")
//        val colorRect = ColorRect()
        val colorLabel = Label("", uiSkin)
        pad(8f)
//        add(colorRect).width(rowHeight).height(rowHeight)
//        add().width(8f)
        add(colorLabel)

        ImContext.cursorRGB.subscribe(this, "Cursor RGB tooltip") {
//            colorRect.color = it.toColor()
            colorLabel.setText(it.joinToString("\n"))

            validate()
            pack()
        }
    }
}