package kr.edoli.imview.ui.panel

import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.Ionicons
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory

class DisplayProfilePanel : Panel(false) {
    val channelButtons = Table()

    init {
        add(Table().apply {
            add(UIFactory.createToggleIconButton(Ionicons.ionMdAperture, ImContext.enableDisplayProfile)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdBrush, ImContext.smoothing)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdSwitch, ImContext.normalize)).width(28f)
        })
        row()
        add(UIFactory.createSlider(Ionicons.ionMdSunny, -10f, 10f, 0.01f, ImContext.imageBrightness))
        row()
        add(UIFactory.createSlider(Ionicons.ionMdContrast, -10f, 10f, 0.01f, ImContext.imageContrast))
        row()
        add(UIFactory.createSlider(Ionicons.ionMdNuclear, 0f, 10f, 0.01f, ImContext.imageGamma))
        row()
        add(channelButtons)
        row()
        add(UIFactory.createSelectBox(ImContext.imageColormap))

        ImContext.mainImage.subscribe { mat ->
            if (mat == null) return@subscribe

            val names = when (mat.channels()) {
                1 -> arrayOf("Gray")
                3 -> arrayOf("Red", "Green", "Blue")
                4 -> arrayOf("Red", "Green", "Blue")
                else -> arrayOf()
            }

            channelButtons.clearChildren()
            channelButtons.add(UIFactory.createToggleTextButton("All", ImContext.imageChannels[0]))
            names.forEachIndexed { i, name ->
                channelButtons.add(UIFactory.createToggleTextButton(name, ImContext.imageChannels[i + 1]))
            }
        }
    }
}