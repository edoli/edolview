package kr.edoli.imview.image

import org.opencv.core.CvType
import org.opencv.core.Mat

data class ImageSpec(val mat: Mat, val maxValue: Double, val numChannels: Int, val bitsPerPixel: Int) {

    companion object {
        fun bitsPerPixel(mat: Mat): Int {
            return when (mat.type()) {
                CvType.CV_8U -> 8
                CvType.CV_8S -> 8
                CvType.CV_16U -> 16
                CvType.CV_16S -> 16
                CvType.CV_32S -> 32
                CvType.CV_32F -> 32
                CvType.CV_64F -> 64
                CvType.CV_8UC1 -> 8
                CvType.CV_8UC2 -> 8
                CvType.CV_8UC3 -> 8
                CvType.CV_8UC4 -> 8
                CvType.CV_8SC1 -> 8
                CvType.CV_8SC2 -> 8
                CvType.CV_8SC3 -> 8
                CvType.CV_8SC4 -> 8
                CvType.CV_16UC1 -> 16
                CvType.CV_16UC2 -> 16
                CvType.CV_16UC3 -> 16
                CvType.CV_16UC4 -> 16
                CvType.CV_16SC1 -> 16
                CvType.CV_16SC2 -> 16
                CvType.CV_16SC3 -> 61
                CvType.CV_16SC4 -> 16
                CvType.CV_32SC1 -> 32
                CvType.CV_32SC2 -> 32
                CvType.CV_32SC3 -> 32
                CvType.CV_32SC4 -> 32
                CvType.CV_32FC1 -> 32
                CvType.CV_32FC2 -> 32
                CvType.CV_32FC3 -> 32
                CvType.CV_32FC4 -> 32
                CvType.CV_64FC1 -> 64
                CvType.CV_64FC2 -> 64
                CvType.CV_64FC3 -> 64
                CvType.CV_64FC4 -> 64
                else -> -1
            }
        }
    }
}