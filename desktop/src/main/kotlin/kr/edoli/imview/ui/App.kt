package kr.edoli.imview.ui

import com.badlogic.gdx.Game
import com.badlogic.gdx.Gdx
import kr.edoli.imview.ImContext
import org.opencv.core.Core

class App : Game() {
    companion object {
        init {
            System.loadLibrary(Core.NATIVE_LIBRARY_NAME)
        }
    }

    override fun create() {
        ImContext.mainFile.subscribe { Gdx.graphics.setTitle(it.name) }
        Gdx.graphics.isContinuousRendering = false

        setScreen(MainScreen())
    }
}