package kr.edoli.edolview.ui.custom

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Skin
import kr.edoli.edolview.image.ClipboardUtils
import kr.edoli.edolview.ui.contextMenu
import kr.edoli.edolview.ui.tooltip
import kr.edoli.edolview.util.format

typealias NumberFormatter = (value: Double) -> String

object Formatters {
    val intFormatter: NumberFormatter = { it.format(0) }
    val decimalFormatter: NumberFormatter = { it.format(2) }
}

class NumberLabel(name: String, skin: Skin, var formatter: NumberFormatter = Formatters.decimalFormatter) : Label("", skin)  {

    var value = 0.0
        set(value) {
            setText(formatter(value))
            field = value
        }


    init {
        tooltip(name)
        contextMenu {
            addMenu("Copy value") {
                ClipboardUtils.putString(value.toString())
            }
            addMenu("Copy text") {
                ClipboardUtils.putString(text.toString())
            }
        }
    }
}