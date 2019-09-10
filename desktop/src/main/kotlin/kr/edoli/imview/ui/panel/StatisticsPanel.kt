package kr.edoli.imview.ui.panel

import com.badlogic.gdx.Gdx
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory
import kr.edoli.imview.ui.custom.NumberLabel
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.forever
import org.opencv.core.Mat
import java.util.concurrent.LinkedBlockingQueue
import kotlin.concurrent.thread
import kotlin.math.sqrt

class StatisticsPanel(imageObservable: NullableObservableValue<Mat>) : Panel(false) {
    val imageQueue = LinkedBlockingQueue<Mat>()

    init {
        val minLabel = NumberLabel("Min value of image", UIFactory.skin)
        val maxLabel = NumberLabel("Max value of image", UIFactory.skin)
        val meanLabel = NumberLabel("Mean value of image", UIFactory.skin)
        val stdLabel = NumberLabel("Standard deviation value of image", UIFactory.skin)


        add("Min").expandX()
        add("Max").expandX()
        add("Mean").expandX()
        add("Std").expandX()

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
                    minLabel.value = minValue
                    maxLabel.value = maxValue
                    meanLabel.value = mean
                    stdLabel.value = standardDeviation
                }
            }
        }

        imageObservable.subscribe { mat ->
            if (mat == null) return@subscribe
            imageQueue.put(mat)
        }
    }
}