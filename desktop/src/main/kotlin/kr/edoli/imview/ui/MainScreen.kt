package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.Screen
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.scenes.scene2d.*
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import com.badlogic.gdx.utils.viewport.ScreenViewport
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.ui.custom.SplitPane
import kr.edoli.imview.ui.panel.FileInfoPanel
import kr.edoli.imview.ui.res.Ionicons
import kr.edoli.imview.ui.res.uiSkin
import kr.edoli.imview.util.reset
import org.lwjgl.opengl.GL30

class MainScreen : Screen {
    val stage = Stage(ScreenViewport(), PolygonSpriteBatch())
    var lastPixelDensity = 0f

    init {
        val imageViewer = ImageViewer()
        val layoutTable = Table().apply {
            setFillParent(true)
        }

        // Top
        layoutTable.add(ToolBar()).height(ToolBar.barHeight + 2f).expandX().fillX()
        layoutTable.row()

        // middle
        val sidePane = object : ScrollPane(SidePanel()) {
            override fun getMinWidth(): Float {
                return 200f
            }
        }.apply {
            setSmoothScrolling(false)
            setOverscroll(false, false)
            setScrollingDisabled(true, false)
        }

        val middleTable = SplitPane(imageViewer, sidePane, false, uiSkin).apply {
            setSplitAmount(1f)
            var isLastCollapsed = false
            addListener(object : InputListener() {
                override fun touchDown(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                    isLastCollapsed = isCollapsed
                    return super.touchDown(event, x, y, pointer, button)
                }
            })
            onSplitChanged = {
                if (it > width - sidePane.minWidth / 2) {
                    ImContext.isShowController.update(false)
                } else {
                    ImContext.isShowController.update(true)
                }
            }
        }

        ImContext.isShowController.subscribe {
            if (middleTable.isCollapsed == !it) {
                return@subscribe
            }
            sidePane.isVisible = it
            middleTable.isCollapsed = !it

            if (it) {
                middleTable.setSplitAmount(1f)
                middleTable.layout()
            } else {
                middleTable.setSplitAmount(1f)
                middleTable.layout()
            }
        }
        layoutTable.add(middleTable).expand().fill()

        // bottom
        layoutTable.row()
        layoutTable.add(StatusBar()).height(StatusBar.barHeight + 2f).expandX().fillX()

        // main stage
        stage.addActor(layoutTable)

        stage.addActor(Window("File info", uiSkin).apply {
            add(FileInfoPanel())
            addListener {
                y = stage.height - height - 32f
                return@addListener false
            }
            titleTable.add().expandX()
            titleTable.add(UIFactory.createIconButton(Ionicons.ionMdClose) {
                ImContext.isShowFileInfo.update(false)
            })

            isResizable = true
            setKeepWithinStage(true)
            ImContext.isShowFileInfo.subscribe { isVisible = it }

        })

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
        val pixelDensity = Gdx.graphics.density
        if (lastPixelDensity != pixelDensity) {
            if (Gdx.graphics.density > 0.75f) {
                (stage.viewport as ScreenViewport).unitsPerPixel = 0.75f / Gdx.graphics.density
                stage.viewport.update(Gdx.graphics.width, Gdx.graphics.height, true)
            } else {
                (stage.viewport as ScreenViewport).unitsPerPixel = 1f
                stage.viewport.update(Gdx.graphics.width, Gdx.graphics.height, true)
            }
            lastPixelDensity = pixelDensity
        }
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