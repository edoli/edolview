package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edoliui.widget.drawable.ColorDrawable
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.WindowClosedMessage
import kr.edoli.imview.bus.WindowOpenMessage
import kr.edoli.imview.res.Colors
import kr.edoli.imview.res.FontAwesomes
import kr.edoli.imview.ui.*
import kr.edoli.imview.ui.screen.ImageListScreen
import kr.edoli.imview.util.WindowUtils
import kr.edoli.imview.util.Windows

/**
 * Created by daniel on 16. 9. 24.
 */
class Toolbar : Table() {

    val iconSize = 32

    val imageListWindowButton = UI.iconButton(FontAwesomes.FaTh, size = iconSize)
    val isShowInfoButton = Context.isShowInfo.checkButton(FontAwesomes.FaTasks, size = iconSize)
    val isFixToolBarButton = Context.isFixToolBar.checkButton(FontAwesomes.FaAnchor, size = iconSize)
    val isShowCrosshairButton = Context.isShowCrosshair.checkButton(FontAwesomes.FaCrosshairs, size = iconSize)
    val comparisonModeButtons = Context.comparisonMode.radioButtons(arrayOf(FontAwesomes.FaPhoto, FontAwesomes.FaShield), size = iconSize)
    val titleLabel = Context.mainPath.label().apply {
        setAlignment(Align.left)
    }

    init {
        background = ColorDrawable(Colors.overlayBackground)

        pad(0f, 24f, 0f, 24f)

        align(Align.left)
        add(titleLabel).expandX().fillX()
        add(UI.optionTable(*comparisonModeButtons))
        add().width(96f)
        add(isShowInfoButton).size(48f).padRight(16f)
        add(isFixToolBarButton).size(48f).padRight(16f)
        add(isShowCrosshairButton).size(48f).padRight(16f)
        add(imageListWindowButton).size(48f)



        imageListWindowButton.onClick {
            ImageListScreen.create()
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