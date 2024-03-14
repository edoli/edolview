package kr.edoli.edolview.ui.component

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.res.uiSkin
import kr.edoli.edolview.util.ObservableValue
import kr.edoli.edolview.util.toColorStr

class RGBTooltip(observable: ObservableValue<DoubleArray>? = null): Table() {

    val colorLabel = Label("", uiSkin)

    init {
        background = uiSkin.getDrawable("tooltip_background")
        pad(8f)
        add(colorLabel)

        if (observable != null) {
            observable.subscribe(this, "Cursor RGB tooltip") {
                updateColor(it)
            }
        }
    }

    fun updateColor(color: DoubleArray) {
        val imageSpec = ImContext.mainImageSpec.get()
        if (imageSpec != null) {
            colorLabel.setText(color.toColorStr(imageSpec, "\n"))
            validate()
            pack()
        }
    }

    fun clearColor() {
        colorLabel.setText("")
    }
}