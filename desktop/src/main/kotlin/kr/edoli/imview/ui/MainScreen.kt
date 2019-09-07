package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.Screen
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Stage
import com.badlogic.gdx.scenes.scene2d.ui.ScrollPane
import kr.edoli.imview.ui.custom.SplitPane
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import com.badlogic.gdx.utils.viewport.ScreenViewport
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.util.reset
import org.lwjgl.opengl.GL30

class MainScreen : Screen {
    val stage = Stage(ScreenViewport(), PolygonSpriteBatch())

    init {
        val imageViewer = ImageViewer()
        val layoutTable = Table().apply {
            setFillParent(true)
        }

        // Top
        layoutTable.add(ToolBar()).height(ToolBar.barHeight + 2f).expandX().fillX()
        layoutTable.row()

        // middle
//        val middleTable = Table().apply {
//            add().expand().fill()
//            add(ScrollPane(SidePanel()).apply {
//                setOverscroll(false, false)
//            }).width(200f).expandY().fillY()
//        }
        val middleTable = SplitPane(null, object : ScrollPane(SidePanel()) {
            init {
                setOverscroll(false, false)
            }

            override fun getMinWidth(): Float {
                return 200f
            }
        }, false, UIFactory.skin).apply {
            splitAmount = 1f
        }
        layoutTable.add(middleTable).expand().fill()

        // bottom
        layoutTable.row()
        layoutTable.add(StatusBar()).height(StatusBar.barHeight + 2f).expandX().fillX()

        stage.addActor(Table().apply {
            setFillParent(true)
            add(imageViewer).expand().fill()
        })
        stage.addActor(layoutTable)

        imageViewer.zIndex = 0
        middleTable.zIndex = 0

        // Keyboard
        stage.addListener(object : InputListener() {
            override fun keyDown(event: InputEvent?, keycode: Int): Boolean {
                if (keycode == Input.Keys.LEFT) {
                    ImContext.prevImage()
                }
                if (keycode == Input.Keys.RIGHT) {
                    ImContext.nextImage()
                }
                if (keycode == Input.Keys.C && UIUtils.ctrl()) {
                    ImContext.marqueeImage.get()?.let { mat ->
                        ClipboardUtils.putImage(mat)
                    }
                }
                if (keycode == Input.Keys.ESCAPE) {
                    ImContext.marqueeBox.update { rect ->
                        rect.reset()
                    }
                }
                if (keycode == Input.Keys.A && UIUtils.ctrl()) {
                    ImContext.mainImage.get()?.let { mat ->
                        ImContext.marqueeBox.update { rect ->
                            rect.x = 0
                            rect.y = 0
                            rect.width = mat.width()
                            rect.height = mat.height()
                            rect
                        }
                    }
                }
                if (keycode == Input.Keys.F4) {
                    stage.isDebugAll = !stage.isDebugAll
                }
                return super.keyDown(event, keycode)
            }
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
        if (Gdx.graphics.density > 0.75f) {
            (stage.viewport as ScreenViewport).unitsPerPixel = 0.75f / Gdx.graphics.density
        }
        stage.viewport.update(width, height, true)
    }

    override fun dispose() {
        stage.dispose()
    }

}