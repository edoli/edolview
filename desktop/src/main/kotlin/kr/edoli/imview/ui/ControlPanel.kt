package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext

class ControlPanel : Panel() {
    init {
        add("View controller")
        row()
        add(Table().apply {
            add(UIFactory.createToggleIconButton(Ionicons.ionMdAperture, ImContext.enableProfile)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdBrush, ImContext.smoothing)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdSwitch, ImContext.normalize)).width(28f)
        })
        row()
        add(UIFactory.createSlider(Ionicons.ionMdSunny, -10f, 10f, 0.01f, ImContext.imageBrightness))
        row()
        add(UIFactory.createSlider(Ionicons.ionMdContrast, -10f, 10f, 0.01f, ImContext.imageContrast))
        row()
        add(UIFactory.createSlider(Ionicons.ionMdNuclear, 0f, 10f, 0.01f, ImContext.imageGamma))

        row().padTop(32f)
        add("Image statistics")
        row()
        add(ImageStatisticsPanel())

        row().padTop(32f)
        add("File information")
        row()
        add(FileInfoPanel())

        ImContext.isShowController.subscribe {
            isVisible = it
        }
    }
}