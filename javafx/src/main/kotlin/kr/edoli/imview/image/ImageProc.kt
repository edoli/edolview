package kr.edoli.imview.image

import javafx.scene.image.Image
import kr.edoli.imview.ImContext
import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc


object ImageProc {
    var lastContrast = 0.0
    var lastBrightness = 0.0
    var lastGamma = 0.0
    var lastPath = ""

    fun process(mat: Mat): Image? {
        val path = ImContext.mainPath.get()
        val contrast = ImContext.imageContrast.get()
        val brightness = ImContext.imageBrightness.get()
        val gamma = ImContext.imageGamma.get()

        if (path == lastPath && contrast == lastContrast && brightness == lastBrightness && gamma == lastGamma) {
            return null
        }
        lastPath = path
        lastContrast = contrast
        lastBrightness = brightness
        lastGamma = gamma


        val newMat = (mat + brightness - 0.5) * (contrast + 1) + 0.5
        Core.pow(newMat, (1 / gamma), newMat)
        Imgproc.threshold(newMat, newMat, 0.0, 0.0, Imgproc.THRESH_TOZERO)

        return ImageConvert.matToImage(newMat)
    }
}