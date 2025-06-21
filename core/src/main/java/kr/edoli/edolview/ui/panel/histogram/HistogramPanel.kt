package kr.edoli.edolview.ui.panel.histogram

import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.channelNames
import kr.edoli.edolview.image.split
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.util.Histogram

class HistogramPanel : Panel(false) {

    val histogramActor = HistogramWidget()
    val buttons = Table()

    init {
        add(histogramActor).expandX().fillX().height(128f)
        row()
        add(buttons).expandX().fillX()

        ImContext.mainImage.subscribe(this, "Update histogram") {
            if (!isGone()) {
                updateHistogram()
            }
        }

        onGoneChanged = { isGone ->
            if (!isGone) {
                updateHistogram()
            }
        }
    }

    fun updateHistogram() {
        val spec = ImContext.mainImageSpec.get() ?: return
        val mat = spec.mat

        val minMax = spec.minMax
        val minValue = minMax.first
        val maxValue = minMax.second

        val numBin = 256

        val histograms = mat.split().map { singleMat ->
            Histogram(numBin).apply {
                computeHistMat(singleMat, minValue, maxValue)
            }
        }

        histogramActor.setData(histograms)

        buttons.clearChildren()
        val names = mat.channelNames()

        (names.indices).forEach { index ->
            buttons.add(UIFactory.createTextButton(names[index]) {
                histogramActor.isShow[index] = it.isChecked
            }.apply {
                isChecked = histogramActor.isShow[index]
                style = UIFactory.textToggleButtonStyle
            })
        }
    }
}