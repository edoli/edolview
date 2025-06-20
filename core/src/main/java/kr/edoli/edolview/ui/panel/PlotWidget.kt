package kr.edoli.edolview.ui.panel

import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.VGWidget
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.ui.vg.ShapeType
import kr.edoli.edolview.ui.vg.SimpleVG
import kr.edoli.edolview.util.toColorValueStr

interface PlotRange {
    operator fun get(index: Int): Float
}

class PlotIntRange(private val range: IntRange): PlotRange {
    override operator fun get(index: Int): Float {
        return (range.first + index * range.step).toFloat()
    }
}

class PlotWidget : VGWidget() {
    var xAxisName: String? = null
    var xAxis: PlotRange? = null
    var data: FloatArray? = null
    var minValue = 0f
    var maxValue = 1f
    var channels = 1

    override fun drawVG(vg: SimpleVG) {
        // grid
        val gridX = (width / 32f).toInt()
        val gridY = (height / 32f).toInt()
        vg.setStrokeColor(Colors.GRID_STROKE)
        vg.grid(0f, 0f, width, height, gridX, gridY)

        val data = this.data
        if (data == null || data.isEmpty()) {
            return
        }

        // draw plot
        val scale = height / (maxValue - minValue)
        val size = data.size / channels
        val step = width / (size - 1)

        val colors = if (channels == 1) {
            arrayOf(Colors.GRAY)
        } else {
            arrayOf(
                Colors.RED,
                Colors.GREEN,
                Colors.BLUE,
                Colors.GRAY
            )
        }

        for (c in 0 until channels) {
            vg.beginPath()
            vg.setStrokeColor(colors[c])

            var offsetX = 0f
            vg.moveTo(offsetX, (data[c] - minValue) * scale)

            for (i in 1 until size) {
                val curY = (data[c + i * channels] - minValue) * scale
                vg.lineTo(offsetX + step, curY)
                offsetX += step
            }

            vg.strokePath()
        }

        if (isOver) {
            val mouseXIndex = (mouseX / step + 0.5f).toInt()

            if (mouseXIndex < 0 || mouseXIndex >= size) {
                return
            }

            // Draw vertical line at mouseXIndex
            vg.setStrokeColor(Colors.VG_TOOLTIP)
            val plotX = mouseXIndex * step
            vg.line(plotX, 0f, plotX, height)

            // Value Tooltip
            val freqWidth = 100f
            val marginFromBar = 8f
            val tooltipHeight = (channels + 1) * 16f + 4f
            val valueX = if (plotX > width - freqWidth) plotX - freqWidth - marginFromBar else plotX + marginFromBar
            val valueY = mouseY.coerceIn(0f, height - tooltipHeight)
            vg.setFillColor(Colors.VG_TOOLTIP_BG)
            vg.rect(valueX, valueY, freqWidth, tooltipHeight, shapeType = ShapeType.STROKE_AND_FILL)

            for (c in 0 until channels) {
                val imageSpec = ImContext.mainImageSpec.get() ?: return
                val value = data[c + mouseXIndex * channels].toColorValueStr(imageSpec, 2)
                vg.setFillColor(colors[c])
                vg.text(value, valueX + 2f, valueY + 16f + 2f + c * 16f)
            }

            xAxis?.let { xAxis ->
                vg.setFillColor(Colors.VG_TOOLTIP)
                vg.text("$xAxisName: ${xAxis[mouseXIndex]}", valueX + 2f, valueY + 16f + 2f + channels * 16f)
            }
        }
    }
}