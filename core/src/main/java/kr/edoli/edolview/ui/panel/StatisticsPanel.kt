package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.Gdx
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.minMax
import kr.edoli.edolview.image.pow
import kr.edoli.edolview.image.split
import kr.edoli.edolview.image.sum
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory.createNumberLabel
import kr.edoli.edolview.util.ObservableLazyValue
import kr.edoli.edolview.util.forever
import org.opencv.core.Mat
import java.util.concurrent.LinkedBlockingQueue
import kotlin.concurrent.thread
import kotlin.math.sqrt

class StatisticsPanel(imageObservable: ObservableLazyValue<Mat?>) : Panel(false) {
    private val imageQueue = LinkedBlockingQueue<Mat>()

    private val minLabel = createNumberLabel(ImContext.statMin)
    private val maxLabel = createNumberLabel(ImContext.statMax)
    private val meanLabel = createNumberLabel(ImContext.statMean)
    private val stdLabel = createNumberLabel(ImContext.statStd)

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

                val vChannel = ImContext.visibleChannel.get() ?: 0
                val subMat = if (vChannel == 0) {
                    mat
                } else {
                    mat.split()[vChannel - 1]
                }

                val num = (mat.total() * if (vChannel == 0) channels else 1).toInt()

                if (num <= 1) {
                    return@forever
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
                    ImContext.statMin.update(minValue * spec.typeMaxValue)
                    ImContext.statMax.update(maxValue * spec.typeMaxValue)
                    ImContext.statMean.update(mean * spec.typeMaxValue)
                    ImContext.statStd.update(standardDeviation * spec.typeMaxValue)
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