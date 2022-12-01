package kr.edoli.edolview.image

import kr.edoli.edolview.ImContext
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc
import java.awt.FileDialog
import java.awt.Frame
import java.io.File
import java.io.IOException
import java.util.*

object MarqueeUtils {

    fun copyImageToClipboard() {
        val mat = croppedImage()
        if (mat != null) {
            ClipboardUtils.putImage(mat)
        }
    }

    fun saveImage() {
        ImContext.mainImage.get() ?: return

        // UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName())
        // val fileChooser = JFileChooser()
        // fileChooser.showSaveDialog(null)

        val frame: Frame? = null
        val fileChooser = FileDialog(frame, "Save file", FileDialog.SAVE)
        fileChooser.directory = File(ImContext.mainPath.get()).absoluteFile.parent
        fileChooser.isVisible = true

        val filePath = fileChooser.file
        val dest = filePath?.let { File(fileChooser.directory + filePath) }

        // val fileChooser = FileChooser()
        // fileChooser.initialDirectory = ImContext.mainFile.get().parentFile
        // fileChooser.title = "Save file"
        // fileChooser.extensionFilters.addAll(
        //         FileChooser.ExtensionFilter("Portable Network Graphics", "*.png"),
        //         FileChooser.ExtensionFilter("JPEG", "*.jpg"),
        //         FileChooser.ExtensionFilter("OpenEXR", "*.exr")
        // )
        // val dest = fileChooser.showSaveDialog(null)
        //
        if (dest != null) {
            try {
                val selectedMat = croppedImage()
                if (selectedMat != null) {
                    val mat = selectedMat.clone()
                    val maxValue = ImContext.mainImageSpec.get()!!.typeMaxValue
                    val factor = when (dest.extension.lowercase(Locale.getDefault())) {
                        "png" -> if (maxValue != -1.0) maxValue else 255.0
                        "tiff" -> 255.0
                        "jpg" -> 255.0
                        else -> 1.0
                    }
                    val numChannels = mat.channels()
                    mat.convertTo(mat, CvType.CV_32FC3, factor)
                    when (numChannels) {
                        3 -> {
                            Imgproc.cvtColor(mat, mat, Imgproc.COLOR_RGB2BGR)
                        }
                        4 -> {
                            Imgproc.cvtColor(mat, mat, Imgproc.COLOR_RGBA2BGRA)
                        }
                    }
                    Imgcodecs.imwrite(dest.absolutePath, mat)
                }
            } catch (ex: IOException) {

            }
        }
    }

    fun boxMeanColor(): DoubleArray {
        if (!ImContext.isValidMarquee) {
            return doubleArrayOf()
        }

        val mainImage = ImContext.mainImage.get()
        val selectBox = ImContext.marqueeBox.get()
        val selectBoxActive = ImContext.marqueeBoxActive.get()

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

    private fun croppedImage(): Mat? {
        if (!ImContext.isValidMarquee) {
            return null
        }

        val mainImage = ImContext.mainImage.get()
        val selectBox = ImContext.marqueeBox.get()
        val selectBoxActive = ImContext.marqueeBoxActive.get()

        return if (mainImage != null) {
            if (selectBoxActive) {
                val mat = mainImage[selectBox]
                mat
            } else {
                mainImage
            }
        } else {
            null
        }
    }
}