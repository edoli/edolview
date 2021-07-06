package kr.edoli.imview.ui.res

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.g2d.NinePatch
import com.badlogic.gdx.graphics.g2d.TextureAtlas
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.utils.Drawable
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import com.badlogic.gdx.scenes.scene2d.utils.TextureRegionDrawable
import kr.edoli.imview.ui.custom.SplitPane

private val uiAtlas = TextureAtlas(Gdx.files.internal("uiskin.atlas"))
private fun res(name: String): Drawable {
    val region = uiAtlas.findRegion(name) ?: throw Error("Not found drawable: $name")
    val splits = region.splits
    return if (splits == null) {
        TextureRegionDrawable(region)
    } else {
        val patch = NinePatch(region, splits[0], splits[1], splits[2], splits[3])
        if (region.pads != null) {
            patch.setPadding(
                    region.pads[0].toFloat(), region.pads[1].toFloat(),
                    region.pads[2].toFloat(), region.pads[3].toFloat())
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
            getDrawable("tooltip_background")))

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

    add("default-horizontal", SplitPane.SplitPaneStyle(
            getDrawable("default-splitpane")
    ))

    add("default-vertical", SplitPane.SplitPaneStyle(
            getDrawable("default-splitpane-vertical")
    ))

    add("default", Window.WindowStyle(
            Font.defaultFont, Colors.normal, getDrawable("default-window")
    ))

    add("dialog", Window.WindowStyle(
            Font.defaultFont, Colors.normal, getDrawable("default-window")
    ).apply { stageBackground = newDrawable("white", 0f, 0f, 0f, 0.45f) })

    add("default-horizontal", Slider.SliderStyle(
            getDrawable("default-slider"),
            getDrawable("default-slider-knob")
    ))

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