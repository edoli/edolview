package kr.edoli.imview.ui

import kr.edoli.imview.ImContext

class ControlPanel : Panel() {
    init {
        add("View controller")
        row()
        add(UIFactory.createToggleTextButton("Smoothing", ImContext.smoothing))
        row()
        add(UIFactory.createToggleTextButton("Normalize", ImContext.normalize))
        row()
        add(UIFactory.createSlider(Ionicons.ionMdSunny, -10f, 10f, 0.1f, ImContext.imageBrightness))
        row()
        add(UIFactory.createSlider(Ionicons.ionMdContrast, -10f, 10f, 0.1f, ImContext.imageContrast))
        row()
        add(UIFactory.createSlider(Ionicons.ionMdNuclear, 0f, 10f, 0.1f, ImContext.imageGamma))

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