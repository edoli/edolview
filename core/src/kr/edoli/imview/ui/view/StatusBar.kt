package kr.edoli.imview.ui.view

import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edoliui.res.FontAwesomes
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.CursorPositionMessage
import kr.edoli.imview.ui.UI
import kr.edoli.imview.ui.onClick
import kr.edoli.imview.util.Clipboard

/**
 * Created by daniel on 16. 9. 23.
 */
class StatusBar : Table() {

    var cursorPositionText = "0,0"
    var cursorRGBText = "0,0,0"
    var boxText = "0,0,0,0"

    init {

        align(Align.left)
        padLeft(8f)
        padRight(8f)

        val cursorPositionLabel = UI.label("($cursorPositionText)")
        val cursorRGBLabel = UI.label("color: ($cursorPositionText)")
        val boxCopyButton = UI.iconButton(FontAwesomes.FaCopy)
        val boxLabel = UI.textField("($boxText)")

        add(cursorPositionLabel).width(128f).padRight(24f)
        add(cursorRGBLabel).width(144f).padRight(24f)
        add(boxCopyButton).size(24f).padRight(8f)
        add(boxLabel)

        boxCopyButton.onClick {
            Clipboard.copy(boxText)
        }

        val t = { q: Byte -> if (q >= 0) q.toInt() else q + 256}


        Context.cursorPosition.subscribe {
            cursorPositionText = "${it.x},${it.y}"
            cursorPositionLabel.setText("($cursorPositionText)")
        }

        Context.cursorRGB.subscribe {
            cursorRGBText = "${t(it[0])}, ${t(it[1])}, ${t(it[2])}"
            cursorRGBLabel.setText("color: ($cursorRGBText)")
        }

        Context.selectBox.subscribe {
            boxText = "${it.x.toInt()},${it.y.toInt()},${it.width.toInt()},${it.height.toInt()}"
            boxLabel.text = "($boxText)"
        }
    }
}