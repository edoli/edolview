package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.ImageConvert
import org.opencv.core.CvType
import kotlin.math.min

class ImageStatisticsPanel : Table() {
    init {
        val minLabel = Label("", UIFactory.skin).tooltip("Min value of image")
        val maxLabel = Label("", UIFactory.skin).tooltip("Max value of image")
        val meanLabel = Label("", UIFactory.skin).tooltip("Mean value of image")
        val varianceLabel = Label("", UIFactory.skin).tooltip("Variance value of image")

        add(minLabel)
        row()
        add(maxLabel)
        row()
        add(meanLabel)
        row()
        add(varianceLabel)

        ImContext.mainImage.subscribe { mat ->
            if (mat == null) return@subscribe

            val channels = mat.channels()
            val num = (mat.total() * channels).toInt()

            val rawData = FloatArray(num)
            mat.get(0, 0, rawData)

            var minValue =  Float.MAX_VALUE
            var maxValue =  Float.MIN_VALUE
            var sum = 0f
            var squareSum = 0f

            rawData.forEach { v ->
                if (v < minValue) minValue = v
                if (v > maxValue) maxValue = v
                sum += v
                squareSum += v * v
            }

            val mean = sum / num
            val variance = (squareSum / num - mean * mean) * (num / (num - 1))

            minLabel.setText(minValue.toString())
            maxLabel.setText(maxValue.toString())
            meanLabel.setText(mean.toString())
            varianceLabel.setText(variance.toString())
        }
    }
}