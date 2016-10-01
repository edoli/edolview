package kr.edoli.imview.ui.screen

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowListener
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.FileDropMessage
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.ColorWidget
import kr.edoli.imview.ui.view.ContextGroup
import kr.edoli.imview.ui.view.ImageViewer
import kr.edoli.imview.ui.view.StatusBar
import kr.edoli.imview.ui.view.Toolbar
import kr.edoli.imview.util.PathManager
import kr.edoli.imview.util.WindowUtils
import kr.edoli.imview.util.Windows

/**
 * Created by daniel on 16. 9. 10.
 */

class MainScreen : BaseScreen() {

    val mainLayout = Table()
    val imageViewer = ImageViewer()

    init {
        val background = ColorWidget(Colors.background)
        background.setFillParent(true)

        mainLayout.setFillParent(true)

        val imageViewWrapper = Table()
        imageViewer.image = Pixmap(Gdx.files.internal("test.png"))
        imageViewWrapper.isTransform = true
        imageViewWrapper.clip = true
        imageViewWrapper.touchable = Touchable.childrenOnly
        imageViewWrapper.add(imageViewer).expand().fill()

        val centerLayout = Table()
        centerLayout.add(imageViewWrapper).expand().fill()

        mainLayout.add(Toolbar()).height(32f).expandX().fillX().row()
        mainLayout.add(ColorWidget(Colors.border)).height(1f).expandX().fillX().row()
        mainLayout.add(centerLayout).expand().fill().row()
        mainLayout.add(ColorWidget(Colors.border)).height(1f).expandX().fillX().row()
        mainLayout.add(StatusBar()).height(32f).expandX().fillX().row()

        val contextGroup = ContextGroup()

        contextGroup.width = stage.width
        contextGroup.height = stage.height



        stage.addActor(background)
        stage.addActor(mainLayout)
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

        val pathManager = PathManager()

        Bus.subscribe(FileDropMessage::class.java) {
            if (windowName == Windows.Main) {
                val path = files[0]
                if (pathManager.isImage(path)) {
                    pathManager.setPath(path)
                    Gdx.app.graphics.setTitle(path)
                    imageViewer.image = Pixmap(Gdx.files.absolute(path))
                }
            }
        }


        stage.root.addListener(object : ClickListener(Input.Buttons.RIGHT) {
            override fun clicked(event: InputEvent?, x: Float, y: Float) {
                contextGroup.menuPosition(x, y)
                contextGroup.show()
            }

            override fun keyDown(event: InputEvent?, keycode: Int): Boolean {
                if (keycode == Input.Keys.LEFT) {
                    val path = pathManager.prev()
                    if (path != null) {
                        Gdx.app.graphics.setTitle(path)
                        imageViewer.image = Pixmap(Gdx.files.absolute(path))
                    }
                }

                if (keycode == Input.Keys.RIGHT) {
                    val path = pathManager.next()
                    if (path != null) {
                        Gdx.app.graphics.setTitle(path)
                        imageViewer.image = Pixmap(Gdx.files.absolute(path))
                    }
                }
                return super.keyDown(event, keycode)
            }
        })
    }

    override fun resize(width: Int, height: Int) {
        super.resize(width, height)
    }
}