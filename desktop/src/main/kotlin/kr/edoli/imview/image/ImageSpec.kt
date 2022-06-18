package kr.edoli.imview.image

import org.opencv.core.CvType
import org.opencv.core.Mat
import kotlin.math.abs

data class ImageSpec(val mat: Mat,
                     val typeMaxValue: Double = abs(mat.typeMax()),
                     val bitsPerPixel: Int = mat.bitsPerPixel()) {
    val numChannels = mat.channels()
    val type = mat.type()
    var isNormalized = false


    fun normalize(): ImageSpec {
        when (mat.channels()) {
            1 -> {
                val alpha = when (mat.type()) {
                    CvType.CV_8U -> 1.0 / 255.0
                    CvType.CV_16U -> 1.0 / 65535.0
                    else -> 1.0
                }
                mat.convertTo(mat, CvType.CV_64FC3, alpha)
            }
            3 -> {
                when {
                    mat.type() == CvType.CV_8UC3 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 255.0)
                    mat.type() == CvType.CV_16UC3 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 65535.0)
                    else -> mat.convertTo(mat, CvType.CV_64FC3)
                }
            }
            4 -> {
                when {
                    mat.type() == CvType.CV_8UC4 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 255.0)
                    mat.type() == CvType.CV_16UC4 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 65535.0)
                    else -> mat.convertTo(mat, CvType.CV_64FC3)
                }
            }
        }
        isNormalized = true

        return this
    }
}