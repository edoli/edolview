package kr.edoli.imview.ui

import com.badlogic.gdx.Game
import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.badlogic.gdx.utils.Timer
import kr.edoli.imview.ImContext
import org.opencv.core.Core
import java.io.File

class App(private val initPath: String?) : Game() {
    companion object {
        init {
            System.loadLibrary(Core.NATIVE_LIBRARY_NAME)
        }
    }

    override fun create() {
        ShaderProgram.pedantic = false
        if (initPath != null) {
            ImContext.mainPath.update(initPath)
        }
        ImContext.mainFileName.subscribe(this, "Update title") { Gdx.graphics.setTitle(it) }
        Gdx.graphics.isContinuousRendering = false

        setScreen(MainScreen())


        // check refresh
        Timer.schedule(object : Timer.Task() {
            override fun run() {
                if (ImContext.autoRefresh.get()) {
                    val file = ImContext.mainFile.get()
                    if (file != null) {
                        val lastModified = ImContext.mainFileLastModified.get()
                        if (file.lastModified() > lastModified) {
                            ImContext.refreshMainPath()
                        }
                    }
                }
            }
        }, 1.0f, 0.2f)
    }
}