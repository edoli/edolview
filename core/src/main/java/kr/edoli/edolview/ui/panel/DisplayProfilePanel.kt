package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Stack
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.ui.res.Ionicons
import kr.edoli.edolview.ui.tooltip

class DisplayProfilePanel : Panel(false) {
    init {
        add(Table().apply {
            add(UIFactory.createToggleIconButton(Ionicons.ionMdAperture, ImContext.enableDisplayProfile)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdBrush, ImContext.smoothing)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdSwitch, ImContext.normalize)).width(28f)
            add(UIFactory.createToggleIconButton(Ionicons.ionMdSync, ImContext.inverse)).width(28f)
        })

        row()

        val minMaxTable = Table().apply {
            val numberFieldMin = UIFactory.createFloatField(ImContext.displayMin)
            val numberFieldMax = UIFactory.createFloatField(ImContext.displayMax)
            val minMaxSwap = UIFactory.createIconButton(Ionicons.ionMdSwap) {
                val displayMin = ImContext.displayMin.get()
                val displayMax = ImContext.displayMax.get()

                ImContext.displayMin.update(displayMax)
                ImContext.displayMax.update(displayMin)
            }
            pad(4f)
            add(numberFieldMin.tooltip("Min value")).expandX().fillX()
            add().width(4f)
            add(minMaxSwap)
            add().width(4f)
            add(numberFieldMax.tooltip("Max value")).expandX().fillX()

            val update = {
                val normalized = ImContext.normalize.get()

                numberFieldMin.isDisabled = normalized
                numberFieldMax.isDisabled = normalized
                minMaxSwap.isDisabled = normalized
                minMaxSwap.touchable = if (normalized) Touchable.disabled else Touchable.enabled

                if (normalized) {
                    val imageMinMax = ImContext.imageMinMax.get()
                    numberFieldMin.text = imageMinMax.first.toString()
                    numberFieldMax.text = imageMinMax.second.toString()
                } else {
                    numberFieldMin.text = ImContext.displayMin.get().toString()
                    numberFieldMax.text = ImContext.displayMax.get().toString()
                }
            }
            ImContext.normalize.subscribe(this@DisplayProfilePanel, "Update display profile") {
                update()
            }
            ImContext.inverse.subscribe(this@DisplayProfilePanel, "Update display profile") {
                update()
            }
            ImContext.imageMinMax.subscribe(this@DisplayProfilePanel, "Update display image min max", false) {
                update()
            }
        }

        add(minMaxTable).expandX().fillX()
        row()
        add(UIFactory.createSlider(Ionicons.ionMdSunny, -10f, 10f, 0.001f, ImContext.imageBrightness)).expandX().fillX().padLeft(4f)
        row()
        add(UIFactory.createSlider(Ionicons.ionMdContrast, -10f, 10f, 0.001f, ImContext.imageExposure)).expandX().fillX().padLeft(4f)
        row()
        add(UIFactory.createSlider(Ionicons.ionMdNuclear, 0f, 10f, 0.001f, ImContext.imageGamma)).expandX().fillX().padLeft(4f)
        row()
        add(Table().apply {
            add(UIFactory.createSelectBox(ImContext.visibleChannel)).padRight(4f)
            add(Stack().apply {
                add(UIFactory.createSelectBox(ImContext.imageMonoColormap).apply {
                    ImContext.visibleChannel.subscribeValue(this@DisplayProfilePanel, "Update display profile") { channel ->
                        isVisible = channel != 0 || ImContext.mainImage.get()?.channels() == 1
                    }
                })
                add(UIFactory.createSelectBox(ImContext.imageRGBColormap).apply {
                    ImContext.visibleChannel.subscribeValue(this@DisplayProfilePanel, "Update display profile") { channel ->
                        val channels = ImContext.mainImage.get()?.channels()
                        isVisible = channel == 0 && (channels == 3 || channels == 4)
                    }
                })
            })
        })
    }

    override fun sizeChanged() {
        super.sizeChanged()
    }
}