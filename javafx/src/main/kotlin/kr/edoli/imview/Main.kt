package kr.edoli.imview

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowAdapter
import kr.edoli.imview.ui.App
import java.io.File

fun main() {
    val config = Lwjgl3ApplicationConfiguration()
    config.useOpenGL3(true, 3, 2)
    config.setWindowListener(object : Lwjgl3WindowAdapter() {
        override fun filesDropped(files: Array<String>) {
            val file = File(files[0])
            if (ImContext.fileManager.isImage(file.name)) {
                ImContext.mainFile.update(file)
            }
        }
    })
    config.setWindowedMode(1280, 720)
    Lwjgl3Application(App(), config)
}