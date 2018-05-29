package kr.edoli.imview.ui

import javafx.geometry.Insets
import javafx.scene.layout.Priority
import javafx.scene.layout.StackPane
import kr.edoli.imview.ImContext
import kr.edoli.imview.util.ClipboardUtils
import kr.edoli.imview.util.bindBoolean
import org.kordamp.ikonli.javafx.FontIcon
import org.kordamp.ikonli.materialdesign.MaterialDesign
import tornadofx.*

class HeaderBar : StackPane() {
    init {
        addClass(Style.overlayPane)

        hbox {
            button("클립보드 확인", FontIcon(MaterialDesign.MDI_CLIPBOARD)) {
                action {
                    ClipboardUtils.showClipboardImage()
                }
            }

            button("이미지 가운데로", FontIcon(MaterialDesign.MDI_TARGET)) {
                action {
                    ImContext.centerImage.onNext(true)
                }
            }

            region {
                hboxConstraints {
                    hGrow = Priority.ALWAYS
                }
            }

            togglebutton("") {
                graphic = FontIcon(MaterialDesign.MDI_CROSSHAIRS_GPS)
                selectedProperty().bindBoolean(ImContext.isShowCrosshair)
            }

            togglebutton("") {
                graphic = FontIcon(MaterialDesign.MDI_CHART_BAR)
                selectedProperty().bindBoolean(ImContext.isShowInfo)
            }

            togglebutton("") {
                graphic = FontIcon(MaterialDesign.MDI_IMAGE_FILTER)
                selectedProperty().bindBoolean(ImContext.isShowConroller)
            }

            padding = Insets(4.0)
        }
    }
}