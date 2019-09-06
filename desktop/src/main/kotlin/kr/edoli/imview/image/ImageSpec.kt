package kr.edoli.imview.image

import org.opencv.core.Mat

data class ImageSpec(val mat: Mat, val maxValue: Double, val numChannels: Int, val bitsPerPixel: Int)