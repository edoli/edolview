package kr.edoli.imview.ui.panel

import com.badlogic.gdx.Gdx
import kr.edoli.imview.image.*
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.custom.NumberLabel
import kr.edoli.imview.util.ObservableValue
import kr.edoli.imview.util.forever
import kr.edoli.imview.util.functionTime
import org.opencv.core.Core
import org.opencv.core.Mat
import java.util.concurrent.LinkedBlockingQueue
import kotlin.concurrent.thread
import kotlin.math.sqrt

class StatisticsPanel(imageObservable: ObservableValue<Mat?>) : Panel(false) {
    val imageQueue = LinkedBlockingQueue<Mat>()

    init {
        val minLabel = NumberLabel("Min value of image", skin)
        val maxLabel = NumberLabel("Max value of image", skin)
        val meanLabel = NumberLabel("Mean value of image", skin)
        val stdLabel = NumberLabel("Standard deviation value of image", skin)


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

                val minMax = mat.minMax()
                val minValue = minMax.first
                val maxValue = minMax.second
                val sum = mat.sum().sum()
                val squareSum = mat.pow(2.0).sum().sum()

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

        imageObservable.subscribe(this, "Statistic queue") { mat ->
            if (mat == null) return@subscribe
            imageQueue.put(mat)
        }
    }
}