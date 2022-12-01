package kr.edoli.edolview.ui.panel.histogram

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.ui.Widget
import kr.edoli.edolview.ui.UIRes
import kr.edoli.edolview.util.Histogram

class HistogramWidget : Widget() {
    val histograms = ArrayList<Histogram>()
    val isShow = ArrayList<Boolean>()
    val colors = arrayOf(Color.RED, Color.GREEN, Color.BLUE, Color.GRAY)

    override fun draw(batch: Batch, parentAlpha: Float) {
        super.draw(batch, parentAlpha)

        while (isShow.size < histograms.size) {
            isShow.add(true)
        }

        histograms.forEachIndexed { histIndex, hist ->
            if (!isShow[histIndex]) {
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