package kr.edoli.imview.ui

import com.badlogic.gdx.Game
import com.badlogic.gdx.Gdx
import kr.edoli.imview.ImContext
import org.opencv.core.Core
import java.io.File

class App(private val args: Array<String>) : Game() {
    companion object {
        init {
            System.loadLibrary(Core.NATIVE_LIBRARY_NAME)
        }
    }

    override fun create() {
        if (args.isNotEmpty()) {
            ImContext.mainFile.update(File(args[0]))
        }
        ImContext.mainFile.subscribe(this, "Update title") { Gdx.graphics.setTitle(it.name) }
        Gdx.graphics.isContinuousRendering = false

        setScreen(MainScreen())
    }
}