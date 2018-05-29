package kr.edoli.imview

import javafx.application.Application
import javafx.scene.Scene
import javafx.scene.input.KeyCode
import javafx.stage.Stage
import kr.edoli.imview.ui.MainPane
import kr.edoli.imview.ui.Style
import kr.edoli.imview.util.ClipboardUtils
import org.opencv.core.Core

class Main: Application() {
    companion object {
        init {
            System.loadLibrary(Core.NATIVE_LIBRARY_NAME)
        }
    }

    override fun start(stage: Stage){
        val root = MainPane()
        val scene = Scene(root,1024.0,512.0)
        scene.stylesheets.add(Style().externalForm)
        stage.scene = scene
        stage.show()

        ImContext.mainPath.subscribe {
            stage.title = it
        }

        scene.setOnKeyPressed { e ->
            when (e.code) {
                KeyCode.LEFT -> ImContext.prevImage()
                KeyCode.RIGHT -> ImContext.nextImage()
                KeyCode.P -> ClipboardUtils.showClipboardImage()
                else -> { }
            }
        }
    }

}

fun main(args: Array<String>){
    Application.launch(Main::class.java, *args)
}