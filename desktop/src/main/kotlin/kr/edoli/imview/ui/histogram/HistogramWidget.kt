package kr.edoli.imview.ui.histogram

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.Actor
import kr.edoli.imview.ui.ColorRect
import kr.edoli.imview.util.Histogram

class HistogramActor : Actor() {
    val histograms = ArrayList<Histogram>()
    val colors = arrayOf(Color.RED, Color.GREEN, Color.BLUE, Color.GRAY)

    override fun draw(batch: Batch, parentAlpha: Float) {
        histograms.forEachIndexed { histIndex, hist ->
            if (!hist.isShow) {
                return@forEachIndexed
            }

            val num = hist.n
            val barWidth = width / (num + 4)
            val freq = hist.freq

            batch.color = colors[histIndex]

            freq.forEachIndexed { index, v ->
                val offset = barWidth * index
                val h = (height * v) / hist.maxFreq
                batch.draw(ColorRect.white, x + offset, y, barWidth, h)
            }
        }
    }
}