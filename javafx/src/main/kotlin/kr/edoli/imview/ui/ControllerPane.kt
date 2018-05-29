package kr.edoli.imview.ui

import javafx.geometry.Insets
import javafx.scene.layout.VBox
import kr.edoli.imview.ImContext
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.util.bindDouble
import tornadofx.*
import javafx.beans.property.SimpleObjectProperty
import javafx.beans.property.ObjectProperty



class ControllerPane : VBox() {

    init {
        addClass(Style.overlayPane)

        // Brightness
        label("Brightness") {
            addClass(Style.controlLabel)
        }
        slider {
            min = -2.0
            max = 2.0
            majorTickUnit = 0.1
            valueProperty().bindDouble(ImContext.imageBrightness)
        }
        pane { prefHeight = 8.0 }

        // Contrast
        label("Contrast") {
            addClass(Style.controlLabel)
        }
        slider {
            min = -1.0
            max = 1.0
            majorTickUnit = 0.1
            valueProperty().bindDouble(ImContext.imageContrast)
        }
        pane { prefHeight = 8.0 }

        // Gamma
        label("Gamma") {
            addClass(Style.controlLabel)
        }
        slider {
            min = 0.0
            max = 4.0
            majorTickUnit = 0.1
            valueProperty().bindDouble(ImContext.imageGamma)
        }
        pane { prefHeight = 16.0 }

        // Animation
        val maxFps = 10.0

        hbox {
            label("Frame speed") {
                addClass(Style.controlLabel)
            }
            spinner(0.0, maxFps) {
                valueFactory.valueProperty().bindDouble(ImContext.frameSpeed)
            }
        }
        slider {
            min = 0.0
            max = maxFps
            majorTickUnit = 1.0
            valueProperty().bindDouble(ImContext.frameSpeed)
        }
        pane { prefHeight = 16.0 }

        button("Clear Cache") {
            action {
                ImageStore.clearCache()
            }
        }

        spacing = 8.0
        padding = Insets(4.0)
    }
}