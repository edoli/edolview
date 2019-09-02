package kr.edoli.imview.image

import javafx.stage.FileChooser
import kr.edoli.imview.ImContext
import kr.edoli.imview.Main
import kr.edoli.imview.main
import kr.edoli.imview.util.ClipboardUtils
import kr.edoli.imview.util.PathManager
import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc
import java.io.File
import java.io.IOException

object SelectBoxUtils {

    fun copyImageToClipboard(doImageProc: Boolean = false) {
        val mat = getSelectedImage(doImageProc)
        if (mat != null) {
            ClipboardUtils.putImage(mat)
        }
    }

    fun saveImage(doImageProc: Boolean = false) {
        ImContext.mainImage.get() ?: return

        val fileChooser = FileChooser()
        fileChooser.initialDirectory = File(ImContext.mainPath.get()).parentFile
        fileChooser.title = "Save file"
        fileChooser.extensionFilters.addAll(
                FileChooser.ExtensionFilter("Portable Network Graphics", "*.png"),
                FileChooser.ExtensionFilter("JPEG", "*.jpg"),
                FileChooser.ExtensionFilter("OpenEXR", "*.exr")
        )
        val dest = fileChooser.showSaveDialog(Main.mainStage)

        if (dest != null) {
            try {
                val selectedMat = getSelectedImage(doImageProc)
                if (selectedMat != null) {
                    val mat = selectedMat.clone()
                    val factor = when (dest.extension.toLowerCase()) {
                        "png" -> 255.0
                        "jpg" -> 255.0
                        else -> 1.0
                    }
                    val numChannels = mat.channels()
                    when (numChannels) {
                        3 -> {
                            Imgproc.cvtColor(mat, mat, Imgproc.COLOR_RGB2BGR)
                        }
                        4 -> {
                            Imgproc.cvtColor(mat, mat, Imgproc.COLOR_RGBA2BGRA)
                        }
                    }
                    Imgcodecs.imwrite(dest.absolutePath, mat * factor)
                }
            } catch (ex: IOException) {

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

    private fun getSelectedImage(doImageProc: Boolean = false): Mat? {
        val mainImage = ImContext.mainImage.get()
        val selectBox = ImContext.selectBox.get()
        val selectBoxActive = ImContext.selectBoxActive.get()

        return if (mainImage != null) {
            if (selectBoxActive) {
                val mat = mainImage[selectBox]
                if (doImageProc) ImageProc.process(mat, false) else mat
            } else {
                if (doImageProc) ImageProc.process(mainImage, false) else mainImage
            }
        } else {
            null
        }
    }
}