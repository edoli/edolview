package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.scenes.scene2d.ui.Label
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.forever
import kr.edoli.imview.util.format
import org.opencv.core.Mat
import java.util.concurrent.LinkedBlockingQueue
import kotlin.concurrent.thread
import kotlin.math.sqrt

class StatisticsPanel(imageObservable: NullableObservableValue<Mat>) : Panel(false) {
    val imageQueue = LinkedBlockingQueue<Mat>()

    init {
        val minLabel = Label("", UIFactory.skin).tooltip("Min value of image")
        val maxLabel = Label("", UIFactory.skin).tooltip("Max value of image")
        val meanLabel = Label("", UIFactory.skin).tooltip("Mean value of image")
        val stdLabel = Label("", UIFactory.skin).tooltip("Variance value of image")

        add("Min")
        add("Max")
        add("Mean")
        add("Std")

        row()

        add(minLabel)
        add(maxLabel)
        add(meanLabel)
        add(stdLabel)

        thread {
            forever {
                val mat = imageQueue.take()

                val channels = mat.channels()
                val num = (mat.total() * channels).toInt()

                val rawData = DoubleArray(num)
                mat.get(0, 0, rawData)

                var minValue = Double.MAX_VALUE
                var maxValue = Double.MIN_VALUE
                var sum = 0.0
                var squareSum = 0.0

                rawData.forEach { v ->
                    if (v < minValue) minValue = v
                    if (v > maxValue) maxValue = v
                    sum += v
                    squareSum += v * v

                    if (imageQueue.isNotEmpty()) {
                        return@forever
                    }
                }

                val mean = sum / num
                val variance = (squareSum / num - mean * mean) * (num / (num - 1))
                val standardDeviation = sqrt(variance)

                Gdx.app.postRunnable {
                    minLabel.setText(minValue.format(2))
                    maxLabel.setText(maxValue.format(2))
                    meanLabel.setText(mean.format(2))
                    stdLabel.setText(standardDeviation.format(2))
                }
            }
        }

        imageObservable.subscribe { mat ->
            if (mat == null) return@subscribe
            imageQueue.put(mat)
        }
    }
}