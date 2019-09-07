package kr.edoli.imview.image

import kr.edoli.imview.ImContext
import org.opencv.core.Mat


object ImageProc {
    var lastEnableProfile = true
    var lastNormalize = false
    var lastPath = ""
    var lastContrast = 0.0
    var lastBrightness = 0.0
    var lastGamma = 0.0

    var cachedMat = Mat()

    fun process(mat: Mat, useCahced: Boolean = true): Mat {
        val enableProfile = ImContext.enableDisplayProfile.get()
        val normalize = ImContext.normalize.get()
        val path = ImContext.mainFile.get()
        val contrast = ImContext.imageContrast.get()
        val brightness = ImContext.imageBrightness.get()
        val gamma = ImContext.imageGamma.get()

        // if (enableProfile == lastEnableProfile && path == lastPath && contrast == lastContrast
        //         && brightness == lastBrightness && gamma == lastGamma && normalize == lastNormalize
        //         && useCahced) {
        //     return cachedMat
        // }
        // lastEnableProfile = enableProfile
        // lastNormalize = normalize
        // lastPath = path
        // lastContrast = contrast
        // lastBrightness = brightness
        // lastGamma = gamma

        cachedMat = mat
        return mat
    }
}