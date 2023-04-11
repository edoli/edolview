package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.Gdx
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.minMax
import kr.edoli.edolview.image.pow
import kr.edoli.edolview.image.split
import kr.edoli.edolview.image.sum
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.custom.NumberLabel
import kr.edoli.edolview.util.ObservableLazyValue
import kr.edoli.edolview.util.forever
import org.opencv.core.Mat
import java.util.concurrent.LinkedBlockingQueue
import kotlin.concurrent.thread
import kotlin.math.sqrt

class StatisticsPanel(imageObservable: ObservableLazyValue<Mat?>) : Panel(false) {
    val imageQueue = LinkedBlockingQueue<Mat>()

    val minLabel = NumberLabel("Min value of image", skin)
    val maxLabel = NumberLabel("Max value of image", skin)
    val meanLabel = NumberLabel("Mean value of image", skin)
    val stdLabel = NumberLabel("Standard deviation value of image", skin)

    init {
        add("Min").expandX()
        add("Max").expandX()

        row()

        add(minLabel)
        add(maxLabel)

        row()

        add("Mean").expandX()
        add("Std").expandX()

        row()

        add(meanLabel)
        add(stdLabel)

        thread {
            forever {
                val mat = imageQueue.take()
                val spec = ImContext.mainImageSpec.get() ?: return@forever

                val channels = mat.channels()
                val num = (mat.total() * channels).toInt()

                if (num <= channels) {
                    return@forever
                }

                val vChannel = ImContext.visibleChannel.get() ?: 0
                val subMat = if (vChannel == 0 ) {
                    mat
                } else {
                    mat.split()[vChannel - 1]
                }

                val minMax = subMat.minMax()
                val minValue = minMax.first
                val maxValue = minMax.second
                val sum = subMat.sum().sum()
                val squareSum = subMat.pow(2.0).sum().sum()

                val mean = sum / num
                val variance = ((squareSum / num) - (mean * mean)) * (num / (num - 1))
                val standardDeviation = sqrt(variance)

                Gdx.app.postRunnable {
                    minLabel.value = minValue * spec.typeMaxValue
                    maxLabel.value = maxValue * spec.typeMaxValue
                    meanLabel.value = mean * spec.typeMaxValue
                    stdLabel.value = standardDeviation * spec.typeMaxValue
                }
            }
        }

        imageObservable.subscribe(this, "Statistic queue") { mat ->
            if (isGone()) {
                return@subscribe
            }
            if (mat == null) return@subscribe
            imageQueue.put(mat)
        }

        ImContext.visibleChannel.subscribeValue(this, "Statistic channel change") {
            if (isGone()) {
                return@subscribeValue
            }
            val mat = imageObservable.get() ?: return@subscribeValue
            imageQueue.put(mat)
        }

        onGoneChanged = {
            if (!it) {
                val mat = imageObservable.get()
                if (mat != null) {
                    imageQueue.put(mat)
                }
            }
        }
    }
}