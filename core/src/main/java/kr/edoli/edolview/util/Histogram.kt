package kr.edoli.edolview.util

import org.opencv.core.Mat
import kotlin.math.max

/**
 * Created by daniel on 16. 2. 27.
 */
class Histogram(val n: Int) {
    val freq = IntArray(n)
    var maxFreq = 0
        private set

    var minValue: Double = 0.0
        private set
    var maxValue: Double = 1.0
        private set


    init {
        clear()
    }

    fun addDataPoint(i: Int) {
        if (i >= n || i < 0) {
            return
        }

        freq[i]++
        if (freq[i] > maxFreq) {
            maxFreq = freq[i]
        }
    }

    fun computeHistMat(mat: Mat, minValue: Double, maxValue: Double) {
        val channels = mat.channels()
        val num = (mat.total() * channels).toInt()

        val rawData = DoubleArray(num)
        mat.get(0, 0, rawData)

        val denominator = (maxValue - minValue)

        rawData.forEach { v ->
            val ind = ((n - 1) * ((v - minValue) / denominator) + 0.5).toInt()
            addDataPoint(ind)
        }

        this.minValue = minValue
        this.maxValue = maxValue
    }

    fun value(index: Int): Double {
        val denominator = (maxValue - minValue)
        if (denominator == 0.0) {
            return 0.0
        }
        return (index * denominator) / (n - 1) + minValue
    }

    fun getFreq(index: Int): Int {
        if (index < 0 || index >= freq.size) {
            return 0
        }
        return freq[index]
    }

    fun clear() {
        for (i in freq.indices) {
            freq[i] = 0
        }
        maxFreq = 0
    }
}
