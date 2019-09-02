package kr.edoli.imview.ui

import javafx.geometry.Insets
import javafx.scene.layout.VBox
import kr.edoli.imview.ImContext
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.util.bindDouble
import tornadofx.*
import javafx.scene.layout.Priority
import kr.edoli.imview.util.ObservableValue
import kr.edoli.imview.util.bindBoolean


class ControllerPane : VBox() {

    init {
        addClass(Style.overlayPane)

        togglebutton {
            text = "Enable profile"
            selectedProperty().bindBoolean(ImContext.enableProfile)
        }

        togglebutton {
            text = "Normalize"
            selectedProperty().bindBoolean(ImContext.normalize)
        }

        // Brightness
        add(ControlItem("Brightness", -2.0, 2.0, ImContext.imageBrightness))

        pane { prefHeight = 8.0 }

        // Contrast
        add(ControlItem("Contrast", -1.0, 1.0, ImContext.imageContrast))

        pane { prefHeight = 8.0 }

        // Gamma
        add(ControlItem("Gamma", 0.0, 4.0, ImContext.imageGamma))
        pane { prefHeight = 16.0 }

        // Rotation
        add(ControlItem("Rotation", 0.0, 360.0, ImContext.rotation))
        pane { prefHeight = 16.0 }

        // Animation
        val maxFps = 60.0

        add(ControlItem("Frame speed", 0.0, maxFps, ImContext.frameSpeed))


        pane { prefHeight = 16.0 }

        button("Clear Cache") {
            action {
                ImageStore.clearCache()
            }
        }

        spacing = 8.0
        padding = Insets(4.0)
    }

    class ControlItem(name: String, minValue: Double, maxValue: Double, property: ObservableValue<Double>) : VBox() {
        var step = 0.1

        init {
            addClass(Style.controlItem)

            hbox {
                spacing = 8.0

                label(name)

                spinner(minValue, maxValue, amountToStepBy = step) {

                    valueFactory.valueProperty().bindDouble(property)
                    hgrow = Priority.ALWAYS
                }
            }

            slider {
                min = minValue
                max = maxValue
                majorTickUnit = 0.1
                valueProperty().bindDouble(property)
            }
        }

    }
}