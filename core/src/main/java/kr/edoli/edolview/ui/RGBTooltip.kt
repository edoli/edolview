package kr.edoli.edolview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.res.uiSkin
import kr.edoli.edolview.util.toColorStr

class RGBTooltip: Table() {

    init {
        background = uiSkin.getDrawable("tooltip_background")
        val colorLabel = Label("", uiSkin)
        pad(8f)
        add(colorLabel)

        ImContext.cursorRGB.subscribe(this, "Cursor RGB tooltip") {
            val imageSpec = ImContext.mainImageSpec.get()
            if (imageSpec != null) {
                colorLabel.setText(it.toColorStr(imageSpec, "\n"))
            }

            validate()
            pack()
        }
    }
}