package kr.edoli.imview.ui.histogram

import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.Slider
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.split
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory
import kr.edoli.imview.util.Histogram

class HistogramPanel : Panel(false) {

    val histogramActor = HistogramWidget()
    val buttons = Table()
    val slider = Slider(1f, 255f, 1f, false, UIFactory.skin).apply {
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

        ImContext.mainImage.subscribe { updateHistogram() }
    }

    fun updateHistogram() {
        val mat = ImContext.mainImage.get() ?: return

        val binSize = slider.value

        val histograms = mat.split().map { singleMat ->
            Histogram(binSize.toInt(), (1 / (binSize - 1)).toDouble()).apply {
                computeHistMat(singleMat)
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
                histograms[index].isShow = it.isChecked
            }.apply {
                isChecked = true
                style = UIFactory.textToggleButtonStyle
            })
        }
    }
}