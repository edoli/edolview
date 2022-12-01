package kr.edoli.edolview.util

import org.opencv.core.Mat

/**
 * Created by daniel on 16. 2. 27.
 */
class Histogram(val n: Int) {
    val freq = IntArray(n)
    var maxFreq = 0
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
            val ind = (n * ((v - minValue) / denominator)).toInt()
            addDataPoint(ind)
        }
    }

    fun clear() {
        for (i in freq.indices) {
            freq[i] = 0
        }
        maxFreq = 0
    }
}
