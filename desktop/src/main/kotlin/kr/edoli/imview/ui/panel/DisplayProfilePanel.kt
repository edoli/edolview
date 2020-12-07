package kr.edoli.imview.ui.panel

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.res.Ionicons
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory
import kr.edoli.imview.ui.res.uiSkin

class DisplayProfilePanel : Panel(false) {
    init {
        add(Table().apply {
            add(UIFactory.createToggleIconButton(Ionicons.ionMdAperture, ImContext.enableDisplayProfile)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdBrush, ImContext.smoothing)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdSwitch, ImContext.normalize)).width(28f)
        })
        row()
        val minMaxTable = Table().apply {
            val numberFieldMin = UIFactory.createNumberField(ImContext.displayMin)
            val numberFieldMax = UIFactory.createNumberField(ImContext.displayMax)
            pad(4f)
            add(Label("Min", uiSkin))
            add().width(4f)
            add(numberFieldMin).expandX().fillX()
            add().width(8f)
            add(Label("Max", uiSkin))
            add().width(4f)
            add(numberFieldMax).expandX().fillX()

            ImContext.normalize.subscribe {
                numberFieldMin.isDisabled = it
                numberFieldMax.isDisabled = it

                if (it) {
                    numberFieldMin.text = ImContext.textureMin.get().toString()
                    numberFieldMax.text = ImContext.textureMax.get().toString()
                } else {
                    numberFieldMin.text = ImContext.displayMin.get().toString()
                    numberFieldMax.text = ImContext.displayMax.get().toString()
                }
            }
        }
        add(minMaxTable).expandX().fillX()
        row()
        add(UIFactory.createSlider(Ionicons.ionMdSunny, -10f, 10f, 0.001f, ImContext.imageBrightness)).expandX().fillX()
        row()
        add(UIFactory.createSlider(Ionicons.ionMdContrast, -10f, 10f, 0.001f, ImContext.imageContrast)).expandX().fillX()
        row()
        add(UIFactory.createSlider(Ionicons.ionMdNuclear, 0f, 10f, 0.001f, ImContext.imageGamma)).expandX().fillX()
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

    override fun sizeChanged() {
        super.sizeChanged()
    }
}