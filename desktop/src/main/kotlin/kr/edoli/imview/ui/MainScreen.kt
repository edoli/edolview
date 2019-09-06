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
        val imageViewer = ImageViewer()
        val layoutTable = Table().apply {
            setFillParent(true)
        }
        // Top
        layoutTable.add(ToolBar()).height(StatusBar.barHeight + 2f).expandX().fillX()
        layoutTable.row()

        // middle
        val middleTable = Table().apply {
            add(imageViewer).expand().fill()
            add(ControlPanel()).width(150f).expandY().fillY()
        }
        layoutTable.add(middleTable).expand().fill()

        // bottom
        layoutTable.row()
        layoutTable.add(StatusBar()).height(StatusBar.barHeight + 2f).expandX().fillX()

        imageViewer.zIndex = 0
        middleTable.zIndex = 0
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
        stage.viewport.update(width, height, true)
    }

    override fun dispose() {
        stage.dispose()
    }

}