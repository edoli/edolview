package kr.edoli.edolview.ui.res

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.g2d.NinePatch
import com.badlogic.gdx.graphics.g2d.TextureAtlas
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.utils.Drawable
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import com.badlogic.gdx.scenes.scene2d.utils.TextureRegionDrawable
import kr.edoli.edolview.ui.custom.SplitPane
import kr.edoli.edolview.ui.drawable.BorderedDrawable
import kr.edoli.edolview.ui.drawable.SliderDrawable
import kr.edoli.edolview.ui.drawable.SplitHandleDrawable

private val uiAtlas = TextureAtlas(Gdx.files.internal("uiskin.atlas"))
private fun res(name: String): Drawable {
    val region = uiAtlas.findRegion(name) ?: throw Error("Not found drawable: $name")
    val splits = region.findValue("split")
    return if (splits == null) {
        TextureRegionDrawable(region)
    } else {
        val patch = NinePatch(region, splits[0], splits[1], splits[2], splits[3])
        val pads = region.findValue("pad")
        if (pads != null) {
            patch.setPadding(
                    pads[0].toFloat(), pads[1].toFloat(),
                    pads[2].toFloat(), pads[3].toFloat())
        }
        NinePatchDrawable(patch)
    }
}

val uiSkin = Skin(uiAtlas).apply {
    val defaultLabelStyle = Label.LabelStyle(Font.defaultFont, Colors.normal)

    add("default-font", Font.defaultFont)
    add("icon-font", Font.ioniconsFont)

    add("default", defaultLabelStyle)
    add("default", TextTooltip.TextTooltipStyle(
            defaultLabelStyle,
            getDrawable("tooltip_background")).apply {
                wrapWidth = 156.0f
    })

    add("default", Button.ButtonStyle(
            getDrawable("default-round"),
            getDrawable("default-round-down"), null))
    add("toggle", Button.ButtonStyle(
            getDrawable("default-round"),
            getDrawable("default-round-down"),
            getDrawable("default-round-down")))

    add("default", TextButton.TextButtonStyle(
            getDrawable("default-round"),
            getDrawable("default-round-down"), null,
            Font.defaultFont).apply {
        disabledFontColor = Colors.inactive
    })
    add("toggle", TextButton.TextButtonStyle(
            getDrawable("default-round"),
            getDrawable("default-round-down"),
            getDrawable("default-round-down"),
            Font.defaultFont).apply {
        disabledFontColor = Colors.inactive
    })

    add("default", ScrollPane.ScrollPaneStyle(
            getDrawable("default-rect"),
            getDrawable("default-scroll"),
            getDrawable("default-round-large"),
            getDrawable("default-scroll"),
            getDrawable("default-round-large")))

    add("default", SelectBox.SelectBoxStyle(
            Font.defaultFont, Colors.normal, SelectBoxDrawable().pad(2f, 5f, 1f, 4f),
            get(ScrollPane.ScrollPaneStyle::class.java),
            List.ListStyle(Font.defaultFont, Colors.normal, Colors.normal, getDrawable("default-select-selection"))
    ))

    val splitHandleMinSize = 6f

    add("default-horizontal", SplitPane.SplitPaneStyle(
            SplitHandleDrawable(Colors.background, Colors.backgroundBorder, Colors.backgroundBorder,
                    splitHandleMinSize, false)
    ).apply {
        handleOver = SplitHandleDrawable(Colors.backgroundOver, Colors.backgroundBorder, Colors.backgroundBorder,
                splitHandleMinSize, false)
    })

    add("default-vertical", SplitPane.SplitPaneStyle(
            SplitHandleDrawable(Colors.background, Colors.backgroundBorder, Colors.backgroundBorder,
                    splitHandleMinSize, true)
    ).apply {
        handleOver = SplitHandleDrawable(Colors.backgroundOver, Colors.backgroundBorder, Colors.backgroundBorder,
                splitHandleMinSize, true)
    })

    add("default", Window.WindowStyle(
            Font.defaultFont, Colors.normal, getDrawable("default-window")
    ))

    add("dialog", Window.WindowStyle(
            Font.defaultFont, Colors.normal, getDrawable("default-window")
    ).apply { stageBackground = newDrawable("white", 0f, 0f, 0f, 0.45f) })

    add("default-horizontal", Slider.SliderStyle(
            SliderDrawable(Colors.backgroundOver),
            BorderedDrawable(Colors.background, Colors.backgroundBorder).apply {
                minWidth = 8f
                minHeight = 16f
            }
    ).apply {
        knobOver = BorderedDrawable(Colors.backgroundOver, Colors.backgroundBorder).apply {
            minWidth = 8f
            minHeight = 16f
        }
    })

    add("default", TextField.TextFieldStyle(
            Font.defaultFont, Colors.normal,
            getDrawable("cursor"),
            getDrawable("selection"),
            getDrawable("textfield")
    ).apply {
        disabledFontColor = Colors.inactive
    })

    add("default", CheckBox.CheckBoxStyle(
            getDrawable("check-on"),
            getDrawable("check-off"),
            Font.defaultFont, Colors.normal
    ))

    add("default", List.ListStyle(
            Font.defaultFont, Colors.normal, Colors.inactive,
            getDrawable("selection")
    ))

    add("default", Tree.TreeStyle(
            getDrawable("tree-plus"),
            getDrawable("tree-minus"),
            getDrawable("default-select-selection")
    ))
}