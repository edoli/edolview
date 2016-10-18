package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.Pixmap
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
        add(cursorRGBLabel).width(172f).padRight(24f)
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
            val textBuilder = StringBuilder()
            for (byte in it) {
                textBuilder.append("${t(byte)}, ")
            }
            cursorRGBText = textBuilder.toString().substring(0, Math.max(textBuilder.length - 2, 0))
            cursorRGBLabel.setText("color: ($cursorRGBText)")
        }

        Context.selectBox.subscribe {
            val y = ((Context.mainImage.get() as Pixmap).height - it.y.toInt() - it.height.toInt())
            boxText = "${it.x.toInt()},${y},${it.width.toInt()},${it.height.toInt()}"
            boxLabel.text = "($boxText)"
        }
    }
}