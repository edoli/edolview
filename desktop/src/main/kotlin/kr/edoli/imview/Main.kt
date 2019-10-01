package kr.edoli.imview

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowAdapter
import com.badlogic.gdx.graphics.glutils.HdpiMode
import com.badlogic.gdx.tools.texturepacker.TexturePacker
import kr.edoli.imview.ui.App
import java.io.File
import kotlin.system.exitProcess


fun main(args: Array<String>) {
    // val setting = TexturePacker.Settings()
    // setting.useIndexes = false
    // setting.maxWidth = 2048
    // setting.maxHeight = 2048
    // TexturePacker.processIfModified(setting, "../../inputdir/ui", ".", "ui")

    val config = Lwjgl3ApplicationConfiguration()
    config.useOpenGL3(true, 3, 2)
    config.setHdpiMode(HdpiMode.Logical)
    config.setWindowListener(object : Lwjgl3WindowAdapter() {
        override fun filesDropped(files: Array<String>) {
            val file = File(files[0])
            if (ImContext.fileManager.isImage(file.name)) {
                ImContext.mainFile.update(file)
            }
        }

        override fun closeRequested(): Boolean {
            exitProcess(0)
        }
    })
    config.setWindowedMode(1280, 720)
    config.setWindowIcon("icon.png")
    Lwjgl3Application(App(), config)
}