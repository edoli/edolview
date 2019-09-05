package kr.edoli.imview.ui

import com.badlogic.gdx.Game
import com.badlogic.gdx.Gdx
import org.opencv.core.Core

class App : Game() {
    companion object {
        init {
            System.loadLibrary(Core.NATIVE_LIBRARY_NAME)
        }
    }

    override fun create() {
        Gdx.graphics.isContinuousRendering = false
        setScreen(MainScreen())
    }
}