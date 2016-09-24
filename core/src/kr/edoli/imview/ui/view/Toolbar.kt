package kr.edoli.imview.ui.view

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowConfiguration
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowListener
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edoliui.res.FontAwesomes
import kr.edoli.imview.BaseApplicationListener
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.WindowClosedMessage
import kr.edoli.imview.bus.WindowOpenMessage
import kr.edoli.imview.ui.UI
import kr.edoli.imview.ui.onClick
import kr.edoli.imview.ui.screen.ImageListScreen
import kr.edoli.imview.util.WindowUtils
import kr.edoli.imview.util.Windows

/**
 * Created by daniel on 16. 9. 24.
 */
class Toolbar : Table() {


    val imageListWindowButton = UI.iconButton(FontAwesomes.FaList)

    init {

        align(Align.left)
        add(imageListWindowButton)

        imageListWindowButton.onClick {
            if (WindowUtils.hasWindow(Windows.ImageList)) {
                return@onClick
            }

            val window = WindowUtils.getWindow(0)
            window.windowListener = object : Lwjgl3WindowListener {
                override fun closeRequested(): Boolean {
                    return true
                }

                override fun focusGained() {
                }

                override fun iconified() {
                }

                override fun deiconified() {
                }

                override fun filesDropped(files: Array<out String>?) {
                    if (files != null) {

                    }
                }

                override fun focusLost() {
                }

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

                override fun filesDropped(files: Array<out String>?) {
                }

                override fun focusLost() {
                }

            })
            WindowUtils.newWindow(Windows.ImageList, BaseApplicationListener(ImageListScreen()), config)
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
        println(WindowUtils.hasWindow(Windows.ImageList))
        if (WindowUtils.hasWindow(Windows.ImageList)) {
            imageListWindowButton.isDisabled = true
        } else {
            imageListWindowButton.isDisabled = false
        }
    }
}