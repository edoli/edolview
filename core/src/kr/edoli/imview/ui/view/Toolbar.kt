package kr.edoli.imview.ui.view

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowConfiguration
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowListener
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edoliui.res.FontAwesomes
import kr.edoli.imview.BaseApplicationListener
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.FileDropMessage
import kr.edoli.imview.bus.WindowClosedMessage
import kr.edoli.imview.bus.WindowOpenMessage
import kr.edoli.imview.ui.*
import kr.edoli.imview.ui.screen.ImageListScreen
import kr.edoli.imview.util.WindowUtils
import kr.edoli.imview.util.Windows
import org.lwjgl.glfw.GLFW
import org.lwjgl.glfw.GLFWDropCallback
import java.awt.Font

/**
 * Created by daniel on 16. 9. 24.
 */
class Toolbar : Table() {


    val imageListWindowButton = UI.iconButton(FontAwesomes.FaTh)
    val isShowCrosshairButton = Context.isShowCrosshair.checkButton(FontAwesomes.FaCrosshairs)
    val comparisonModeButtons = Context.comparisonMode.radioButtons(arrayOf(FontAwesomes.FaImage, FontAwesomes.FaAdjust))
    val titleLabel = Context.mainPath.label()

    init {

        pad(0f, 8f, 0f, 8f)

        align(Align.left)
        add(imageListWindowButton).size(24f)
        add(isShowCrosshairButton).size(24f)
        add().width(72f)
        comparisonModeButtons.forEach { add(it).size(24f) }
        add().expandX()
        add(titleLabel)



        imageListWindowButton.onClick {
            if (WindowUtils.hasWindow(Windows.ImageList)) {
                return@onClick
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

        Bus.subscribe(WindowClosedMessage::class.java) {
            checkWindows()
        }

        Bus.subscribe(WindowOpenMessage::class.java) {
            checkWindows()
        }

    }

    fun checkWindows() {
        if (WindowUtils.hasWindow(Windows.ImageList)) {
            imageListWindowButton.isDisabled = true
        } else {
            imageListWindowButton.isDisabled = false
        }
    }
}