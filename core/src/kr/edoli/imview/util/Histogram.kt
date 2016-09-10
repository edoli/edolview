package kr.edoli.imview.util

/**
 * Created by daniel on 16. 2. 27.
 */
class Histogram(n: Int) {
    val freq: IntArray
    var maxFreq: Int = 0
        private set

    init {
        freq = IntArray(n)
        clear()
    }

    fun addDataPoint(i: Int) {
        freq[i]++
        if (freq[i] > maxFreq) {
            maxFreq = freq[i]
        }
    }

    val number: Int
        get() = freq.size

    fun clear() {
        freq.map { 0 }
        maxFreq = 0
    }
}
