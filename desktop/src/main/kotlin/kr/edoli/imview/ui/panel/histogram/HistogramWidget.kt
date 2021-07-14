package kr.edoli.imview.ui.panel.histogram

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.ui.Widget
import kr.edoli.imview.ui.UIRes
import kr.edoli.imview.util.Histogram

class HistogramWidget : Widget() {
    val histograms = ArrayList<Histogram>()
    val colors = arrayOf(Color.RED, Color.GREEN, Color.BLUE, Color.GRAY)

    override fun draw(batch: Batch, parentAlpha: Float) {
        super.draw(batch, parentAlpha)

        histograms.forEachIndexed { histIndex, hist ->
            if (!hist.isShow) {
                return@forEachIndexed
            }

            val num = hist.n
            val barWidth = width / num
            val freq = hist.freq

            batch.color = colors[histIndex]

            freq.forEachIndexed { index, v ->
                val offset = barWidth * index
                val h = (height * v) / hist.maxFreq
                batch.draw(UIRes.white, x + offset, y, barWidth, h)
            }
        }
    }
}