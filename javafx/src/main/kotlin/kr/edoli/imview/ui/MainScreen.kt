package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Screen
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.scenes.scene2d.Stage
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.viewport.ScreenViewport
import org.lwjgl.opengl.GL30

class MainScreen : Screen {
    val stage = Stage(ScreenViewport(), PolygonSpriteBatch())

    init {
        val layoutTable = Table().apply {
            setFillParent(true)
        }

        layoutTable.add(ImageViewer().apply {
            Gdx.app.postRunnable {
                stage.scrollFocus = this
            }
        }).expand().fill()
        layoutTable.add(ControlPanel()).width(150f).expandY().fillY()

        stage.addActor(layoutTable)
    }

    override fun hide() {
    }

    override fun show() {
        Gdx.input.inputProcessor = stage
    }

    override fun render(delta: Float) {
        Gdx.gl.glClearColor(0f, 0f, 0f, 1f)
        Gdx.gl.glClear(GL30.GL_COLOR_BUFFER_BIT)

        stage.act(delta)

        stage.draw()
    }

    override fun pause() {
    }

    override fun resume() {
    }

    override fun resize(width: Int, height: Int) {
        stage.viewport.update(width, height)
    }

    override fun dispose() {
        stage.dispose()
    }

}