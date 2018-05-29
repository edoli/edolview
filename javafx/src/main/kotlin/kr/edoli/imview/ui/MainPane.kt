package kr.edoli.imview.ui

import javafx.scene.input.TransferMode
import javafx.scene.layout.*
import javafx.scene.shape.Rectangle
import kr.edoli.imview.ImContext
import kr.edoli.imview.util.bindBoolean
import tornadofx.*

class MainPane : VBox() {
    private val imageViewer = ImageViewer()

    init {
        val rect = Rectangle()
        val imageViewPane = Pane(imageViewer)
        imageViewer.prefWidthProperty().bind(imageViewPane.widthProperty())
        imageViewer.prefHeightProperty().bind(imageViewPane.heightProperty())
        rect.widthProperty().bind(imageViewPane.widthProperty())
        rect.heightProperty().bind(imageViewPane.heightProperty())

        imageViewer.clip = rect

        val controllerPane = ControllerPane().apply {
            visibleProperty().bindBoolean(ImContext.isShowConroller)
        }
        val fileInfoPane = FileInfoPane().apply {
            visibleProperty().bindBoolean(ImContext.isShowInfo)
        }
        val headerBar = HeaderBar()
        val statusBar = StatusBar()

        val centerPane = StackPane().apply {

            add(imageViewPane)
            vbox {
                isPickOnBounds = false

                add(headerBar)
                hbox {
                    isPickOnBounds = false
                    vboxConstraints { vGrow = Priority.ALWAYS }

                    add(fileInfoPane)
                    region {
                        isPickOnBounds = false
                        hboxConstraints { hGrow = Priority.ALWAYS }
                    }
                    add(controllerPane)
                }
            }

            setOnDragOver { e ->
                val db = e.dragboard
                if (db.hasFiles()) {
                    e.acceptTransferModes(TransferMode.COPY)
                } else {
                    e.consume()
                }
            }

            setOnDragDropped { e ->
                val db = e.dragboard
                var success = false
                if (db.hasFiles()) {
                    success = true
                    for (file in db.files) {
                        val filePath = file.absolutePath

                        ImContext.mainPath.update(filePath)
                    }
                }
                e.isDropCompleted = success
                e.consume()
            }
        }

        add(centerPane.apply {
            vboxConstraints { vGrow = Priority.ALWAYS }
        })
        add(statusBar)
    }
}