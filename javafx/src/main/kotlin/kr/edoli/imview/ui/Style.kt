package kr.edoli.imview.ui

import javafx.scene.Cursor
import javafx.scene.paint.Color
import javafx.scene.text.Font
import javafx.scene.text.FontWeight
import tornadofx.*

class Style : Stylesheet() {

    companion object {
        val overlayPane by cssclass()
        val controlItem by cssclass()
        val boldLabel by cssclass()
    }

    init {
        Font.loadFont(javaClass.getResource("/SpoqaHanSansBold.ttf").toExternalForm(), 24.0)
        Font.loadFont(javaClass.getResource("/SpoqaHanSansRegular.ttf").toExternalForm(), 24.0)

        root {
            fontFamily = "SpoqaHanSans-Regular"
        }

        button {
            backgroundColor += c("#FFFFFF")

            and(hover) {
                backgroundColor += c("#ecfefd")
                cursor = Cursor.HAND
            }
        }

        overlayPane {
            backgroundColor += c("#00000088")

            label {
                textFill = Color.WHITE
            }

            boldLabel {
                fontFamily = "SpoqaHanSans-Bold"
                textFill = Color.LIGHTGRAY
            }
        }

        controlItem {
            label {
                textFill = Color.WHITE
            }

            spinner {
                textField {
                    padding = box(2.px)
                }
                incrementArrowButton {
                }
            }
        }

        track {
            prefHeight = 2.px
            backgroundColor += Color.WHITE
        }

        thumb {
            prefWidth = 20.px
            prefHeight = 20.px
            backgroundColor += c("#0984e3")
        }
    }
}