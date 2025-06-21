package kr.edoli.edolview.ui.panel

import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.split
import kr.edoli.edolview.ui.VGWidget
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.ui.vg.ShapeType
import kr.edoli.edolview.ui.vg.SimpleVG
import kr.edoli.edolview.util.toColorValueStr
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat

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
    val isShow = ArrayList<Boolean>()

    val maxValues = ArrayList<Float>()
    val minValues = ArrayList<Float>()
    var values: FloatArray? = null
    var numChannel = 0

    fun setData(mat: Mat?) {
        values = null
        maxValues.clear()
        minValues.clear()

        if (mat == null || mat.empty()) return

        // CV_32F 변환
        val mat32f = if (mat.type() == CvType.CV_32F) mat else {
            Mat().also { mat.convertTo(it, CvType.CV_32F) }
        }

        val channels = mat32f.channels()

        while (isShow.size < channels) {
            isShow.add(true)
        }

        val splitMats = mat32f.split()
        splitMats.forEach { subMat ->
            val minMax = Core.minMaxLoc(subMat)
            maxValues.add(minMax.maxVal.toFloat())
            minValues.add(minMax.minVal.toFloat())
            subMat.release()
        }

        val size = mat32f.total().toInt()
        values = FloatArray(size * channels)
        mat32f.get(0, 0, values)
        numChannel = channels

        if (mat32f !== mat) {
            mat32f.release()
        }
    }

    override fun drawVG(vg: SimpleVG) {
        // grid
        val gridX = (width / 32f).toInt()
        val gridY = (height / 32f).toInt()
        vg.setStrokeColor(Colors.GRID_STROKE)
        vg.grid(0f, 0f, width, height, gridX, gridY)

        val values = this.values
        if (values == null || values.isEmpty()) {
            return
        }

        val maxValue = maxValues.filterIndexed { index, _ -> isShow[index] }.maxOrNull()
        val minValue = minValues.filterIndexed { index, _ -> isShow[index] }.minOrNull()

        if (maxValue == null || minValue == null || maxValue < minValue) {
            return
        }

        // draw plot
        val scale = height / (maxValue - minValue)
        val size = values.size / numChannel
        val step = width / (size - 1)

        val colors = if (numChannel == 1) {
            arrayOf(Colors.GRAY)
        } else {
            arrayOf(
                Colors.RED,
                Colors.GREEN,
                Colors.BLUE,
                Colors.GRAY
            )
        }

        for (c in 0 until numChannel) {
            if (!isShow[c]) {
                continue
            }
            vg.beginPath()
            vg.setStrokeColor(colors[c])

            var offsetX = 0f
            vg.moveTo(offsetX, (values[c] - minValue) * scale)

            for (i in 1 until size) {
                val curY = (values[c + i * numChannel] - minValue) * scale
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
            val tooltipHeight = (numChannel + 1) * 16f + 4f
            val valueX = if (plotX > width - freqWidth) plotX - freqWidth - marginFromBar else plotX + marginFromBar
            val valueY = mouseY.coerceIn(0f, height - tooltipHeight)
            vg.setFillColor(Colors.VG_TOOLTIP_BG)
            vg.rect(valueX, valueY, freqWidth, tooltipHeight, shapeType = ShapeType.STROKE_AND_FILL)

            for (c in 0 until numChannel) {
                val imageSpec = ImContext.mainImageSpec.get() ?: return
                val value = values[c + mouseXIndex * numChannel].toColorValueStr(imageSpec, 2)
                vg.setFillColor(colors[c])
                vg.text(value, valueX + 2f, valueY + 16f + 2f + c * 16f)
            }

            xAxis?.let { xAxis ->
                vg.setFillColor(Colors.VG_TOOLTIP)
                vg.text("$xAxisName: ${xAxis[mouseXIndex]}", valueX + 2f, valueY + 16f + 2f + numChannel * 16f)
            }
        }
    }
}