package kr.edoli.imview.image

import kr.edoli.imview.ImContext
import kr.edoli.imview.util.ClipboardUtils
import org.opencv.core.Core

object SelectBoxUtils {
    fun copyImageToClipboard() {
        val mainImage = ImContext.mainImage.get()
        val selectBox = ImContext.selectBox.get()
        val selectBoxActive = ImContext.selectBoxActive.get()

        if (mainImage != null) {
            if (selectBoxActive) {
                ClipboardUtils.putImage(mainImage[selectBox])
            } else {
                ClipboardUtils.putImage(mainImage)
            }
        }
    }

    fun selectBoxMeanColor(): DoubleArray {
        val mainImage = ImContext.mainImage.get()
        val selectBox = ImContext.selectBox.get()
        val selectBoxActive = ImContext.selectBoxActive.get()

        if (mainImage != null) {
            return if (selectBoxActive) {
                val meanVal = Core.mean(mainImage[selectBox])
                if (mainImage.channels() == 3 || mainImage.channels() == 4) {
                    meanVal.`val`
                } else {
                    meanVal.`val`.sliceArray(0..0)
                }
            } else {
                val meanVal = Core.mean(mainImage)
                if (mainImage.channels() == 3 || mainImage.channels() == 4) {
                    meanVal.`val`
                } else {
                    meanVal.`val`.sliceArray(0..0)
                }
            }
        }
        return doubleArrayOf(0.0)
    }
}