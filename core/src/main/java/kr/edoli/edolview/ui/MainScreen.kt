package kr.edoli.edolview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Screen
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Cursor
import com.badlogic.gdx.graphics.GL30
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.graphics.glutils.ShapeRenderer
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Stage
import com.badlogic.gdx.scenes.scene2d.ui.ScrollPane
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.ui.Window
import com.badlogic.gdx.utils.viewport.ScreenViewport
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.config.KeyboardShortcuts
import kr.edoli.edolview.ui.custom.SplitPane
import kr.edoli.edolview.ui.panel.FileInfoPanel
import kr.edoli.edolview.ui.res.Ionicons
import kr.edoli.edolview.ui.res.uiSkin
import kr.edoli.edolview.ui.window.ObservableInfo
import kr.edoli.edolview.util.FullScreenManager
import kotlin.math.max

class MainScreen : Screen {
    val stage = Stage(ScreenViewport(), PolygonSpriteBatch())
    private var lastUIScale = 0f

    init {
        val imageViewer = ImageViewer()
        val layoutTable = Table().apply {
            setFillParent(true)
        }

        // Top
        layoutTable.add(ToolBar()).height(ToolBar.barHeight + 2f).expandX().fillX().minWidth(0f)
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

            onSplitChanged = {
                if (it > width - sidePane.minWidth / 2) {
                    ImContext.isShowController.update(false)
                } else {
                    ImContext.isShowController.update(true)
                }
            }
        }

        ImContext.isShowController.subscribe(this, "Layout") {
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
        layoutTable.add(middleTable).expand().fill().minWidth(0f)

        // bottom
        layoutTable.row()
        val statusBar = StatusBar()
        val statusBarCell = layoutTable.add(statusBar)
        statusBarCell.expandX().fillX().minWidth(0f)

        ImContext.isShowStatusBar.subscribe(this, "Statusbar visibility") {
            statusBar.isVisible = it
            if (it) {
                statusBarCell.height(StatusBar.barHeight + 2f)
            } else {
                statusBarCell.height(0f)
            }
            layoutTable.invalidate()
        }

        // main stage
        stage.addActor(layoutTable)

        stage.addActor(Window("File info", uiSkin).apply {
            add(FileInfoPanel()).expand().fill()
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
            ImContext.isShowFileInfo.subscribe(this@MainScreen, "Layout") { isVisible = it }

        })

        val posA = Vector2()
        val posB = Vector2()

        val shapeRenderer = ShapeRenderer()
        val thickness = 4.0f

        stage.addActor(object : Actor() {
            fun drawRect(actor: Actor, color: Color) {
                posA.set(0.0f, 0.0f)
                posB.set(actor.width, actor.height)

                actor.localToStageCoordinates(posA)
                actor.localToStageCoordinates(posB)

                shapeRenderer.color = color
                shapeRenderer.rectLine(posA.x, posA.y, posA.x, posB.y, thickness)
                shapeRenderer.rectLine(posA.x, posA.y, posB.x, posA.y, thickness)
                shapeRenderer.rectLine(posB.x, posA.y, posB.x, posB.y, thickness)
                shapeRenderer.rectLine(posA.x, posB.y, posB.x, posB.y, thickness)
            }

            override fun draw(batch: Batch, parentAlpha: Float) {
                super.draw(batch, parentAlpha)

                if (ImContext.presentationMode.get()) {
                    batch.end()

                    shapeRenderer.begin(ShapeRenderer.ShapeType.Filled)
                    shapeRenderer.transformMatrix = batch.transformMatrix
                    shapeRenderer.projectionMatrix = batch.projectionMatrix

                    stage.keyboardFocus?.let { drawRect(it, Color.RED) }
                    presentationFocus?.let { drawRect(it, Color.GREEN) }

                    shapeRenderer.end()

                    batch.begin()
                }
            }
        })

        imageViewer.zIndex = 0
        middleTable.zIndex = 0

        // Keyboard
        stage.addListener(object : InputListener() {
            val fullScreenManager = FullScreenManager()

            override fun keyDown(event: InputEvent, keycode: Int): Boolean {
                // Debug keys
                if (KeyboardShortcuts.SHOW_OBSERVABLE_INFO.check(keycode)) {
                    ObservableInfo.show()
                }

                if (KeyboardShortcuts.SHOW_DEBUG_UI.check(keycode)) {
                    stage.isDebugAll = !stage.isDebugAll
                }

                if (KeyboardShortcuts.REFRESH_ASSET.check(keycode)) {
                    // refresh
                    ImContext.refreshAsset()
                }

                // UI Scaling
                if (KeyboardShortcuts.UI_SCALE_DOWN.check(keycode)) {
                    ImContext.uiScale.update { max(it - 0.25f, 0.5f) }
                }

                if (KeyboardShortcuts.UI_SCALE_UP.check(keycode)) {
                    ImContext.uiScale.update { it + 0.25f }
                }

                if (KeyboardShortcuts.PRESENTATION_MODE_TOGGLE.check(keycode)) {
                    ImContext.presentationMode.update { !it }
                }

                if (KeyboardShortcuts.RGB_TOOLTIP_TOGGLE.check(keycode)) {
                    ImContext.isShowRGBTooltip.update { !it }
                }

                if (KeyboardShortcuts.FULLSCREEN_TOGGLE.check(keycode)) {
                    fullScreenManager.toggle()
                }

                if (KeyboardShortcuts.LOAD_FROM_CLIPBOARD.check(keycode)) {
                    ImContext.loadFromClipboard()
                }
                return super.keyDown(event, keycode)
            }
        })
        stage.addCaptureListener(object : InputListener() {
            override fun enter(event: InputEvent?, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
                Gdx.graphics.setSystemCursor(Cursor.SystemCursor.Arrow)
                super.enter(event, x, y, pointer, fromActor)
            }
        })
    }

    override fun hide() {
    }

    override fun show() {
        Gdx.input.inputProcessor = stage
    }

    override fun render(delta: Float) {
        val uiScale = ImContext.uiScale.get()
        if (lastUIScale != uiScale) {
            (stage.viewport as ScreenViewport).unitsPerPixel = 1.25f / uiScale
            stage.viewport.update(Gdx.graphics.width, Gdx.graphics.height, true)
            lastUIScale = uiScale
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