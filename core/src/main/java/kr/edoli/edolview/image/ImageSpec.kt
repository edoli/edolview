package kr.edoli.edolview.image

import org.opencv.core.CvType
import org.opencv.core.Mat
import kotlin.math.abs

data class ImageSpec(val mat: Mat,
                     val typeMaxValue: Double = abs(mat.typeMax()),
                     val bitsPerPixel: Int = mat.bitsPerPixel()) {

    val numChannels = mat.channels()
    val type = mat.type()
    val isInt = (mat.type() % 8 < 5)

    private var _minMax: Pair<Double, Double>? = null
    val minMax: Pair<Double, Double>
        get() {
            assert(isNormalized)

            if (_minMax == null) {
                _minMax = mat.minMax()
            }
            return _minMax!!
        }

    var isNormalized = false

    fun normalize(): ImageSpec {

        val alpha = when (mat.depth()) {
            0 -> 1.0 / 255.0
            1 -> 1.0 / 127.0
            2 -> 1.0 / 65535.0
            3 -> 1.0 / 32767.0
            else -> 1.0
        }

        mat.convertTo(mat, CvType.CV_64F, alpha)

        isNormalized = true

        return this
    }
}