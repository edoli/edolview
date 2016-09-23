package kr.edoli.imview.ui.screen

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Screen
import com.badlogic.gdx.graphics.GL20
import com.badlogic.gdx.scenes.scene2d.Stage
import com.badlogic.gdx.utils.viewport.ScreenViewport

/**
 * Created by daniel on 16. 9. 10.
 */

open class BaseScreen : Screen {
    val stage = Stage(ScreenViewport())

    override fun show() {
        Gdx.input.inputProcessor = stage
    }

    override fun pause() {

    }

    override fun resize(width: Int, height: Int) {
        stage.viewport.update(width, height)
    }

    override fun hide() {
    }

    override fun render(delta: Float) {
        stage.act()

        Gdx.gl.glClear(GL20.GL_COLOR_BUFFER_BIT or GL20.GL_DEPTH_BUFFER_BIT or if (Gdx.graphics.bufferFormat.coverageSampling) GL20.GL_COVERAGE_BUFFER_BIT_NV else 0)


        stage.draw()
    }

    override fun resume() {
    }

    override fun dispose() {
    }

}