package kr.edoli.imview.ui

import com.badlogic.gdx.Game
import com.badlogic.gdx.Gdx
import kr.edoli.imview.ImContext
import org.opencv.core.Core
import java.io.File

class App(val args: Array<String>) : Game() {
    companion object {
        init {
            System.loadLibrary(Core.NATIVE_LIBRARY_NAME)
        }
    }

    override fun create() {
        val f = File("test.txt")
        val writer = f.writer()
        args.forEach { writer.append(it) }
        writer.close()

        if (args.isNotEmpty()) {
            ImContext.mainFile.update(File(args[0]))
        }
        ImContext.mainFile.subscribe { Gdx.graphics.setTitle(it.name) }
        Gdx.graphics.isContinuousRendering = false

        setScreen(MainScreen())
    }
}