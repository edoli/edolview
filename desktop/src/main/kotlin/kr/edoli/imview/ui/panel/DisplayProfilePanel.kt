package kr.edoli.imview.ui.panel

import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.res.Ionicons
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
        add(Table().apply {
            add(UIFactory.createSelectBox(ImContext.visibleChannel)).padRight(4f)
            add(UIFactory.createSelectBox(ImContext.imageColormap).apply {
                ImContext.visibleChannel.subscribe { channel ->
                    isVisible = channel != 0 || ImContext.mainImage.get()?.channels() == 1
                }
            })
        })
    }
}