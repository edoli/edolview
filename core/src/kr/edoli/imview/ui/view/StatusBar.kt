package kr.edoli.imview.ui.view

import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edoliui.res.FontAwesomes
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.CursorPositionMessage
import kr.edoli.imview.bus.SelectBoxMessage
import kr.edoli.imview.ui.UI
import kr.edoli.imview.ui.onClick
import kr.edoli.imview.util.Clipboard

/**
 * Created by daniel on 16. 9. 23.
 */
class StatusBar : Table() {

    var cursorPositionText = "0,0"
    var boxText = "0,0,0,0"

    init {

        align(Align.left)
        padLeft(8f)
        padRight(8f)

        val cursorPositionLabel = UI.label("($cursorPositionText)")
        val boxCopyButton = UI.iconButton(FontAwesomes.FaCopy)
        val boxLabel = UI.textField("($boxText)")

        add(cursorPositionLabel).width(128f).padRight(24f)
        add(boxCopyButton).size(24f).padRight(8f)
        add(boxLabel)

        boxCopyButton.onClick {
            Clipboard.copy(boxText)
        }

        Bus.subscribe(CursorPositionMessage::class.java, {
            cursorPositionText = "$x,$y"
            cursorPositionLabel.setText("($cursorPositionText)")
        })

        Bus.subscribe(SelectBoxMessage::class.java, {
            boxText = "$x,$y,$width,$height"
            boxLabel.text = "($boxText)"
        })
    }
}