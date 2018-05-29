package kr.edoli.imview.ui

import javafx.geometry.Insets
import javafx.geometry.Pos
import javafx.scene.control.Label
import javafx.scene.layout.*
import javafx.scene.shape.Rectangle
import kr.edoli.imview.ImContext
import kr.edoli.imview.util.floor
import kr.edoli.imview.util.toColor
import kr.edoli.imview.util.toColorStr
import tornadofx.hboxConstraints

class StatusBar : Pane() {

    val mousePosText = Label()
    val rectText = Label()
    val cursorRGBText = Label()
    val cursorRGBRect = Rectangle()
    val selecBoxRGBText = Label()
    val selecBoxRGBRect = Rectangle()
    val zoomText = Label()

    init {
        ImContext.cursorPosition.subscribe(this) {
            mousePosText.text = "Cursor: (${it.x.floor().toInt()}, ${it.y.floor().toInt()})"
        }

        ImContext.selectBox.subscribe(this) {
            rectText.text = "Selection: (${it.x}, ${it.y}, ${it.width}, ${it.height})"
        }

        val boxSize = 20.0

        cursorRGBRect.width = boxSize
        cursorRGBRect.height = boxSize
        ImContext.cursorRGB.subscribe(this) {
            val imageSpec = ImContext.imageSpec.get()
            if (imageSpec != null) {
                cursorRGBText.text = "Color: (${it.toColorStr(imageSpec.maxValue)})"
                cursorRGBRect.fill = it.toColor()
            }
        }

        selecBoxRGBRect.width = boxSize
        selecBoxRGBRect.height = boxSize
        ImContext.selectBoxRGB.subscribe(this) {
            val imageSpec = ImContext.imageSpec.get()
            if (imageSpec != null) {
                selecBoxRGBText.text = "Mean color: (${it.toColorStr(imageSpec.maxValue)})"
                selecBoxRGBRect.fill = it.toColor()
            }
        }

        ImContext.zoomRatio.subscribe(this) {
            zoomText.text = "${(it * 100).toInt()}%"
        }

        val hBox = HBox()
        hBox.children.addAll(
                mousePosText,
                rectText,
                cursorRGBText,
                cursorRGBRect,
                selecBoxRGBText,
                selecBoxRGBRect,
                Region().apply {
                    hboxConstraints { hGrow = Priority.ALWAYS }
                },
                zoomText
        )
        hBox.spacing = 12.0
        hBox.padding = Insets(0.0, 4.0, 0.0, 4.0)
        hBox.prefWidthProperty().bind(widthProperty())
        hBox.alignment = Pos.CENTER_LEFT

        children.add(hBox)
    }
}