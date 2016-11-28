package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.scenes.scene2d.ui.Image
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edoliui.widget.drawable.ColorDrawable
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.ColorCopyMessage
import kr.edoli.imview.bus.SelectionCopyMessage
import kr.edoli.imview.res.Colors
import kr.edoli.imview.res.FontAwesomes
import kr.edoli.imview.ui.UI
import kr.edoli.imview.ui.label
import kr.edoli.imview.ui.onClick
import kr.edoli.imview.util.Clipboard
import java.util.*

/**
 * Created by daniel on 16. 9. 23.
 */
class StatusBar : Table() {

    var cursorPositionText = "0,0"
    var cursorRGBText = "0,0,0"
    var boxText = "0,0,0,0"

    init {
        background = ColorDrawable(Colors.statusBarBackground)

        align(Align.left)
        padLeft(8f)
        padRight(8f)

        val cursorPositionLabel = UI.label("position: ($cursorPositionText)")
        val boxCopyButton = UI.iconButton(FontAwesomes.FaCopy, size = 22)
        val boxLabel = UI.textField("selection: ($boxText)")
        val cursorRGBLabel = UI.label("color: ($cursorRGBText)")
        val colorTile = ColorTile()
        val zoomLabel = Context.zoomRate.label({it -> "%.2f%%".format(it * 100)})

        add(cursorPositionLabel).width(196f).padRight(24f)
        add(boxCopyButton).size(24f).padRight(8f)
        add(boxLabel).width(256f)
        add(cursorRGBLabel).width(192f).padRight(24f)
        add(colorTile)
        add().expand()
        add(zoomLabel)

        boxCopyButton.onClick {
            Clipboard.copy(boxText)
        }


        Context.cursorPosition.subscribe {
            cursorPositionText = "${it.x},${it.y}"
            cursorPositionLabel.setText("position: ($cursorPositionText)")
        }

        Context.cursorRGB.subscribe {
            val textBuilder = StringBuilder()
            for (value in it) {
                textBuilder.append("$value, ")
            }
            cursorRGBText = textBuilder.toString().substring(0, Math.max(textBuilder.length - 2, 0))
            cursorRGBLabel.setText("color: ($cursorRGBText)")
        }

        Context.selectBox.subscribe {
            boxText = "${it.x.toInt()},${it.y.toInt()},${it.width.toInt()},${it.height.toInt()}"
            boxLabel.text = "selection: ($boxText)"
        }


        Bus.subscribe(ColorCopyMessage::class.java) {
            val rgb = Context.cursorRGB.get()
            if (rgb.size == 1) {
                colorTile.addColor(Color(rgb[0] / 255f, rgb[0] / 255f, rgb[0] / 255f, 1f))
            } else if (rgb.size == 3) {
                colorTile.addColor(Color(rgb[0] / 255f, rgb[1] / 255f, rgb[2] / 255f, 1f))
            } else if (rgb.size == 4) {
                colorTile.addColor(Color(rgb[0] / 255f, rgb[1] / 255f, rgb[2] / 255f, rgb[3] / 255f))
            }
        }
    }

    class ColorTile : Table() {

        val colors = ArrayList<Color>()
        var needRefresh = false

        init {
            (0..5).forEach { add().size(32f) }
        }

        fun refresh() {
            clearChildren()

            colors.forEach {
                add(Image(ColorDrawable(it))).size(32f)
            }

            for (i in 0..5-colors.size) {
                add().size(32f)
            }
        }

        fun addColor(color: Color) {
            while (colors.size >= 6) {
                colors.removeAt(colors.lastIndex)
            }
            colors.add(0, color)
            needRefresh = true
        }

        override fun act(delta: Float) {
            if (needRefresh) {
                refresh()
                needRefresh = false
            }
            super.act(delta)
        }
    }
}