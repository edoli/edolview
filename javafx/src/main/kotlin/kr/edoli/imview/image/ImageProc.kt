package kr.edoli.imview.image

import javafx.scene.image.Image
import kr.edoli.imview.ImContext
import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc


object ImageProc {
    var lastEnableProfile = true
    var lastNormalize = false
    var lastPath = ""
    var lastContrast = 0.0
    var lastBrightness = 0.0
    var lastGamma = 0.0

    var cachedMat = Mat()

    fun process(mat: Mat, useCahced: Boolean = true): Mat {
        val enableProfile = ImContext.enableProfile.get()
        val normalize = ImContext.normalize.get()
        val path = ImContext.mainPath.get()
        val contrast = ImContext.imageContrast.get()
        val brightness = ImContext.imageBrightness.get()
        val gamma = ImContext.imageGamma.get()

        if (enableProfile == lastEnableProfile && path == lastPath && contrast == lastContrast
                && brightness == lastBrightness && gamma == lastGamma && normalize == lastNormalize
                && useCahced) {
            return cachedMat
        }
        lastEnableProfile = enableProfile
        lastNormalize = normalize
        lastPath = path
        lastContrast = contrast
        lastBrightness = brightness
        lastGamma = gamma

        return if (enableProfile) {

            val newMat = ((if (normalize) mat.normalize() else mat) + brightness - 0.5) * (contrast + 1) + 0.5
            Core.pow(newMat, (1 / gamma), newMat)
            Imgproc.threshold(newMat, newMat, 0.0, 0.0, Imgproc.THRESH_TOZERO)
            cachedMat = newMat
            newMat
        } else {
            cachedMat = mat
            mat
        }
    }
}