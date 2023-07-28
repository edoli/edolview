package kr.edoli.edolview.util

import org.opencv.core.CvType

fun Int.cvPresion(depth: Int): Int {
    val numChannels = CvType.channels(this)
    return CvType.makeType(depth, numChannels)
}

fun Int.cvChannels(numChannels: Int): Int {
    val depth = CvType.depth(this)
    return CvType.makeType(depth, numChannels)
}

fun String.toCvDepth(): Int {
    return when (this) {
        "float64" -> CvType.CV_64F
        "float32" -> CvType.CV_32F
        "float16" -> CvType.CV_16F
        "uint16" -> CvType.CV_16U
        "uint8" -> CvType.CV_8U
        "int32" -> CvType.CV_32S
        "int16" -> CvType.CV_16S
        "int8" -> CvType.CV_8S
        else -> -1
    }
}

fun String.toCvType(numChannels: Int): Int {
    return when (this) {
        "float64" -> CvType.CV_64FC(numChannels)
        "float32" -> CvType.CV_32FC(numChannels)
        "float16" -> CvType.CV_16FC(numChannels)
        "uint16" -> CvType.CV_16UC(numChannels)
        "uint8" -> CvType.CV_8UC(numChannels)
        "int32" -> CvType.CV_32SC(numChannels)
        "int16" -> CvType.CV_16SC(numChannels)
        "int8" -> CvType.CV_8SC(numChannels)

        "bool" -> CvType.CV_8UC(numChannels)
        else -> -1
    }
}