package kr.edoli.imview.ui

import javafx.scene.control.Button
import javafx.scene.effect.DropShadow
import javafx.scene.layout.StackPane
import javafx.scene.paint.Color
import javafx.scene.shape.Circle

class CircleButton(radius: Double) : Button() {
    init {
        val dropShadow = DropShadow()
        dropShadow.offsetX = 2.0
        dropShadow.offsetY = 2.0

        style = "-fx-background-radius: 5em; " +
                "-fx-min-width: ${radius * 2}px; " +
                "-fx-min-height: ${radius * 2}px; " +
                "-fx-max-width: ${radius * 2}px; " +
                "-fx-max-height: ${radius * 2}px;"

        effect = dropShadow
        isCache = true
    }
}