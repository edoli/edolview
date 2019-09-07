package kr.edoli.imview.util

import org.opencv.core.Mat

/**
 * Created by daniel on 16. 2. 27.
 */
class Histogram(val n: Int, val binSize: Double) {
    var isShow = true
    val freq: IntArray
    var maxFreq: Int = 0
        private set

    init {
        freq = IntArray(n)
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

    fun computeHistMat(mat: Mat) {
        val channels = mat.channels()
        val num = (mat.total() * channels).toInt()

        val rawData = DoubleArray(num)
        mat.get(0, 0, rawData)

        rawData.forEach { v ->
            val ind = (v / binSize).toInt()
            addDataPoint(ind)
        }
    }

    val number: Int
        get() = freq.size

    fun clear() {
        freq.map { 0 }
        maxFreq = 0
    }
}
