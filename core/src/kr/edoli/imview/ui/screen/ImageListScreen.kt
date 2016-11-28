package kr.edoli.imview.ui.screen

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowConfiguration
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowListener
import com.badlogic.gdx.scenes.scene2d.ui.ScrollPane
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.BaseApplicationListener
import kr.edoli.imview.ComparisonMetric
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.FileDropMessage
import kr.edoli.imview.bus.WindowClosedMessage
import kr.edoli.imview.bus.WindowOpenMessage
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.ColorWidget
import kr.edoli.imview.ui.UI
import kr.edoli.imview.ui.radioButtons
import kr.edoli.imview.ui.view.ImageListViewer
import kr.edoli.imview.util.WindowUtils
import kr.edoli.imview.util.Windows

/**
 * Created by daniel on 16. 9. 24.
 */
class ImageListScreen : BaseScreen() {

    companion object {
        fun create() {
            if (WindowUtils.hasWindow(Windows.ImageList)) {
                return
            }

            val config = Lwjgl3WindowConfiguration()
            config.setTitle("Image List")
            config.setWindowListener(object : Lwjgl3WindowListener {
                override fun closeRequested(): Boolean {
                    WindowUtils.removeWindow(Windows.ImageList)
                    Bus.send(WindowClosedMessage(Windows.ImageList))
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
                    Bus.send(FileDropMessage(Windows.ImageList, files as Array<String>))
                }

                override fun focusLost() {
                }

            })
            WindowUtils.newWindow(Windows.ImageList, BaseApplicationListener(ImageListScreen::class.java), config)
            Bus.send(WindowOpenMessage(Windows.ImageList))
        }
    }

    val imageListViewer = ImageListViewer()
    val scrollPane = ScrollPane(imageListViewer)

    init {
        val background = ColorWidget(Colors.background)
        background.setFillParent(true)

        imageListViewer.setFillParent(true)

        val imageListToolBar = Table()
        val buttons = Context.comparisonMetric.radioButtons(
                arrayOf("PSNR", "SSIM", "RMSE", "MSE"), ComparisonMetric.values, isIcon = false)
        imageListToolBar.add(UI.optionTable(*buttons))

        val mainTable = Table()
        mainTable.setFillParent(true)

        mainTable.add(imageListToolBar).height(42f).expandX().fillX().align(Align.left).row()
        mainTable.add(scrollPane).expand().fill()


        stage.addActor(background)
        stage.addActor(mainTable)
    }

    override fun resize(width: Int, height: Int) {
        super.resize(width, height)

        scrollPane.invalidate()
        imageListViewer.invalidate()
    }
}