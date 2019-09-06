package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext

class ControlPanel : Panel() {
    init {
        align(Align.top)

        add(CollapsiblePanel("View controller", Table().apply {
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
        })).fillX()

        row()
        addHorizontalDivider().pad(4f)
        row()

        add(CollapsiblePanel("Image statistics", StatisticsPanel(ImContext.mainImage))).fillX()

        row()
        addHorizontalDivider().pad(4f)
        row()

        add(CollapsiblePanel("Selection statistics", StatisticsPanel(ImContext.marqueeImage))).fillX()

        row()
        addHorizontalDivider().pad(4f)
        row()

        add(CollapsiblePanel("File information", FileInfoPanel())).fillX()

        ImContext.isShowController.subscribe {
            isVisible = it
        }
    }
}