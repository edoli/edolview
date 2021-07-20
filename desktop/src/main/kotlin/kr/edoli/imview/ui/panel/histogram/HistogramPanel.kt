package kr.edoli.imview.ui.panel.histogram

import com.badlogic.gdx.Input
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.Slider
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.minMax
import kr.edoli.imview.image.split
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory
import kr.edoli.imview.util.Histogram
import org.opencv.core.CvType

class HistogramPanel : Panel(false) {

    val histogramActor = HistogramWidget()
    val buttons = Table()
    val slider = Slider(0f, 1f, 0.01f, false, skin).apply {
        setButton(Input.Buttons.LEFT)
        value = 100f
        addListener(object : ChangeListener() {
            override fun changed(event: ChangeEvent, actor: Actor) {
                updateHistogram()
            }
        })
    }

    init {
        add(histogramActor).expandX().fillX().height(128f)
        row()
        add(buttons).expandX().fillX()
        row()
        add(slider).expandX().fillX()

        ImContext.mainImage.subscribe(this, "Update histogram") { updateHistogram() }
    }

    fun updateHistogram() {
        val spec = ImContext.mainImageSpec.get() ?: return
        val mat = spec.mat

        var minValue = 0.0
        var maxValue = 1.0

        if (!CvType.isInteger(spec.type)) {
            val minMax = ImContext.imageMinMax.get()
            minValue = minMax.first
            maxValue = minMax.second
        }

        val numBin = 256

        val histograms = mat.split().map { singleMat ->
            Histogram(numBin).apply {
                computeHistMat(singleMat, minValue, maxValue)
            }
        }

        histogramActor.histograms.clear()
        histogramActor.histograms.addAll(histograms)

        buttons.clearChildren()
        val names = when (mat.channels()) {
            1 -> arrayOf("Gray")
            3 -> arrayOf("Red", "Green", "Blue")
            4 -> arrayOf("Red", "Green", "Blue", "Alpha")
            else -> arrayOf()
        }

        (names.indices).forEach { index ->
            buttons.add(UIFactory.createTextButton(names[index]) {
                histogramActor.isShow[index] = it.isChecked
            }.apply {
                isChecked = true
                style = UIFactory.textToggleButtonStyle
            })
        }
    }
}