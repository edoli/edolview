package kr.edoli.imview.ui.screen

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowListener
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.actions.Actions
import com.badlogic.gdx.scenes.scene2d.ui.Image
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.FileDropMessage
import kr.edoli.imview.res.Colors
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.ui.ColorWidget
import kr.edoli.imview.ui.view.*
import kr.edoli.imview.util.PathManager
import kr.edoli.imview.util.WindowUtils
import kr.edoli.imview.util.Windows

/**
 * Created by daniel on 16. 9. 10.
 */

class MainScreen : BaseScreen() {

    val mainLayout = Table()
    val imageViewer = ImageViewer()

    val pathManager = PathManager()


    var infoViewShow = true
    var toolBarShow = true

    val toolBar = Toolbar()
    val infoView = InfoView()

    init {
        val background = ColorWidget(Colors.background)
        background.setFillParent(true)

        mainLayout.setFillParent(true)

        if (Context.args.get().isEmpty()) {
            Context.mainImage.update(ImageStore.get(ImageStore.Where.Internal, "test.jpg"))
        } else {
            val path = Context.args.get()[0]
            updateImageFromPath(path)
            pathManager.setPath(path)
        }

        val imageViewWrapper = Table()
        imageViewWrapper.isTransform = true
        imageViewWrapper.clip = true
        imageViewWrapper.touchable = Touchable.childrenOnly
        imageViewWrapper.add(imageViewer).expand().fill()

        val centerLayout = Table()
        centerLayout.add(imageViewWrapper).expand().fill()

        val statusBarHeight = 32f

        mainLayout.add(centerLayout).expand().fill().row()
        mainLayout.add(StatusBar()).height(statusBarHeight).expandX().fillX().row()



        val overlayTable = object : Table() {
            override fun layout() {
                super.layout()

                toolBarShow = true
                infoViewShow = true
            }
        }
        overlayTable.setFillParent(true)
        overlayTable.add(toolBar).height(64f).colspan(3).expandX().fillX().row()
        overlayTable.add(infoView).width(280f).expandY().fillY()
        overlayTable.add().expand()
        overlayTable.add().row()
        overlayTable.add().height(32f).colspan(3).expandX().fillX()

        Context.isShowInfo.subscribe {
            infoView.isVisible = it
        }

        Context.isFixToolBar.subscribe {
            if (it) {
                showInfoView()
                showToolBar()
            }
        }



        val contextGroup = ContextGroup()

        contextGroup.width = stage.width
        contextGroup.height = stage.height



        stage.addActor(background)
        stage.addActor(mainLayout)
        stage.addActor(overlayTable)
        stage.addActor(contextGroup)


        val window = WindowUtils.getWindow(0)
        window.windowListener = object : Lwjgl3WindowListener {
            override fun closeRequested(): Boolean {
                WindowUtils.closeAllWindow()
                return true
            }

            override fun focusGained() {
            }

            override fun iconified() {
            }

            override fun deiconified() {
            }

            @Suppress("UNCHECKED_CAST")
            override fun filesDropped(files: Array<out String>?) {
                Bus.send(FileDropMessage(Windows.Main, files as Array<String>))
            }

            override fun focusLost() {
            }
        }

        Bus.subscribe(FileDropMessage::class.java) {
            if (windowName == Windows.Main) {
                val path = files[0]
                if (pathManager.isImage(path)) {
                    updateImageFromPath(path)
                    pathManager.setPath(path)
                }
            }
        }


        stage.root.addListener(object : ClickListener(Input.Buttons.RIGHT) {

            var windowWidth = 0
            var windowHeight = 0

            override fun clicked(event: InputEvent?, x: Float, y: Float) {
                contextGroup.menuPosition(x, y)
                contextGroup.show()
            }

            override fun keyDown(event: InputEvent?, keycode: Int): Boolean {
                if (keycode == Input.Keys.LEFT) {
                    updateImageFromPath(pathManager.prev())
                }

                if (keycode == Input.Keys.RIGHT) {
                    updateImageFromPath(pathManager.next())
                }

                if (keycode == Input.Keys.F11) {
                    if (Gdx.graphics.isFullscreen) {
                        Gdx.graphics.setWindowedMode(windowWidth, windowHeight)
                    } else {
                        windowWidth = Gdx.graphics.width
                        windowHeight = Gdx.graphics.height

                        Gdx.graphics.setFullscreenMode(Gdx.graphics.displayMode)
                    }
                }
                return super.keyDown(event, keycode)
            }

            override fun mouseMoved(event: InputEvent?, x: Float, y: Float): Boolean {
                if (x < 72f) showInfoView() else hideInfoView()

                if (!infoViewShow) {
                    if (y > stage.height - 72f) showToolBar() else hideToolBar()
                }

                return super.mouseMoved(event, x, y)
            }
        })
    }

    fun showToolBar() {
        if (!toolBarShow) {
            toolBarShow = true
            toolBar.addAction(Actions.moveTo(toolBar.x, stage.height - toolBar.height, 0.2f))
        }
    }

    fun hideToolBar() {
        if (Context.isFixToolBar.get()) {
            return
        }
        if (toolBarShow) {
            toolBarShow = false
            toolBar.addAction(Actions.moveTo(toolBar.x, stage.height, 0.2f))
        }
    }

    fun showInfoView() {
        showToolBar()
        if (!infoViewShow) {
            infoViewShow = true
            infoView.addAction(Actions.moveTo(0f, infoView.y, 0.2f))
        }
    }

    fun hideInfoView() {
        if (Context.isFixToolBar.get()) {
            return
        }
        if (infoViewShow) {
            infoViewShow = false
            infoView.addAction(Actions.moveTo(-infoView.width, infoView.y, 0.2f))
        }
    }

    fun updateImageFromPath(path: String?) {
        if (path != null && Gdx.files.absolute(path).exists()) {
            Gdx.app.graphics.setTitle(path)
            val pixmap = ImageStore.get(ImageStore.Where.Absolute, path)
            Context.mainImage.update(pixmap)
            Context.mainPath.update(path)
        }
    }

    override fun resize(width: Int, height: Int) {
        super.resize(width, height)

        infoViewShow = true
        toolBarShow = true

    }
}