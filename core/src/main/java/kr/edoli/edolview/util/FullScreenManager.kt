package kr.edoli.edolview.util

import com.badlogic.gdx.Gdx

class FullScreenManager {
    var windowWidth = Gdx.graphics.width
    var windowHeight = Gdx.graphics.height

    fun toggle() {
        if (Gdx.graphics.isFullscreen) {
            Gdx.graphics.setWindowedMode(windowWidth, windowHeight)
        } else {
            windowWidth = Gdx.graphics.width
            windowHeight = Gdx.graphics.height
            Gdx.graphics.setFullscreenMode(Gdx.graphics.displayMode)
        }
    }
}