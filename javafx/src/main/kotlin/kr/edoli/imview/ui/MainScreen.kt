package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Screen
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.Stage
import com.badlogic.gdx.scenes.scene2d.ui.Skin
import com.badlogic.gdx.scenes.scene2d.ui.Slider
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import com.badlogic.gdx.utils.viewport.ScreenViewport
import kr.edoli.imview.ImContext
import org.lwjgl.opengl.GL30

class MainScreen : Screen {
    val stage = Stage(ScreenViewport(), PolygonSpriteBatch())

    val skin = Skin(Gdx.files.internal("uiskin.json"))

    init {
        stage.addActor(ImageViewer())

        val style = skin.get("default-horizontal", Slider.SliderStyle::class.java)

        stage.addActor(Table().apply {
            y = 30f
            x = 100f
            add(Slider(-10f, 10f, 0.1f, false, style).apply {
                value = ImContext.imageBrightness.get()
                addListener(object : ChangeListener() {
                    override fun changed(event: ChangeEvent, actor: Actor) {
                        val value = this@apply.value
                        ImContext.imageBrightness.update(value)
                    }
                })
            })
            row()
            add(Slider(0f, 10f, 0.1f, false, style).apply {
                value = ImContext.imageGamma.get()
                addListener(object : ChangeListener() {
                    override fun changed(event: ChangeEvent, actor: Actor) {
                        val value = this@apply.value
                        ImContext.imageGamma.update(value)
                    }
                })
            })
        })
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