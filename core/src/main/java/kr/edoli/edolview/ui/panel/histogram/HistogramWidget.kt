package kr.edoli.edolview.ui.panel.histogram

import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.VGWidget
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.ui.vg.ShapeType
import kr.edoli.edolview.ui.vg.SimpleVG
import kr.edoli.edolview.util.Histogram
import kr.edoli.edolview.util.toColorValueStr

class HistogramWidget : VGWidget() {
    val histograms = ArrayList<Histogram>()
    val isShow = ArrayList<Boolean>()
    val colors = arrayOf(Colors.RED, Colors.GREEN, Colors.BLUE, Colors.GRAY)

    fun setData(histograms: List<Histogram>) {
        this.histograms.clear()
        this.histograms.addAll(histograms)

        while (isShow.size < histograms.size) {
            isShow.add(true)
        }
    }

    override fun drawVG(vg: SimpleVG) {
        // grid
        val gridX = (width / 32f).toInt()
        val gridY = (height / 32f).toInt()
        vg.setStrokeColor(Colors.GRID_STROKE)
        vg.grid(0f, 0f, width, height, gridX, gridY)

        if (histograms.isEmpty()) {
            return
        }

        // draw histogram
        val num = histograms[0].n
        val barWidth = width / num

        histograms.forEachIndexed { histIndex, hist ->

            if (!isShow[histIndex]) {
                return@forEachIndexed
            }
            val freq = hist.freq


            vg.beginPath()
            vg.setFillColor(colors[histIndex])
            vg.moveTo(0f, 0f)

            var lastH = -1f
            freq.forEachIndexed { index, v ->
                val offset = barWidth * index
                val h = (height * v) / hist.maxFreq

                if (lastH != h) {
                    vg.lineTo(offset, h)
                }
                vg.lineTo(offset + barWidth, h)

                lastH = h
            }

            vg.lineTo(width, 0f)
            vg.fillPath()
        }

        if (isOver) {
            val mouseXIndex = (mouseX / barWidth).toInt()

            if (mouseXIndex < 0 || mouseXIndex >= num) {
                return
            }

            // Draw vertical line at mouseXIndex
            vg.setStrokeColor(Colors.VG_TOOLTIP)
            val barX = (mouseXIndex + 0.5f) * barWidth
            vg.line(barX, 0f, barX, height)

            // Value Tooltip
            val freqWidth = 64f
            val marginFromBar = 8f
            val valueX = if (barX > width - freqWidth) barX - freqWidth - marginFromBar else barX + marginFromBar
            val valueY = mouseY.coerceIn(0f, height - 20f)
            vg.setFillColor(Colors.VG_TOOLTIP_BG)
            vg.rect(valueX, valueY, freqWidth, 20f, shapeType = ShapeType.STROKE_AND_FILL)

            val imageSpec = ImContext.mainImageSpec.get() ?: return
            val histValue = histograms[0].value(mouseXIndex).toColorValueStr(imageSpec, 2)
            vg.setFillColor(Colors.VG_TOOLTIP)
            vg.text("X: $histValue", valueX + 2f, valueY + 16f + 2f)

        }
    }
}