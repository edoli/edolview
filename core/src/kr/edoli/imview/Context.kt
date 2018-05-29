package kr.edoli.imview

import com.badlogic.gdx.math.GridPoint2
import kr.edoli.imview.image.ImageProc
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.ObservableValue
import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.core.Rect
import org.opencv.core.Scalar

/**
 * Created by daniel on 16. 10. 2.
 */
object Context {
    val args = ObservableValue(arrayOf<String>())

    val mainImage = NullableObservableValue<Mat>(null)
    val selectedImage = NullableObservableValue<Mat>(null)

    val mainPath = ObservableValue("")

    val cursorPosition = ObservableValue(GridPoint2())
    val cursorRGB = ObservableValue(floatArrayOf())
    val selectBox = ObservableValue(Rect())
    val zoomBox = ObservableValue(Rect())
    val zoomRate = ObservableValue(1f)

    val comparisonMode = ObservableValue(ComparisonMode.Diff)
    val comparisonMetric = ObservableValue(ComparisonMetric.PSNR)

    val isShowCrosshair = ObservableValue(true)
    val isFixToolBar = ObservableValue(true)
    val isShowInfo = ObservableValue(false)

    val imageContrast = ObservableValue(1f)
    val imageBrightness = ObservableValue(1f)

    fun processedImage(): Mat? {
        val currentImage = mainImage.get()

        return if (currentImage != null) {
            val contrast = imageContrast.get()
            val brightness = imageBrightness.get()

            var resultImage = currentImage.clone()

            val scale = contrast
            val offset = (brightness - 1)

            Core.add(resultImage, Scalar(offset.toDouble(), offset.toDouble(), offset.toDouble(), 1.0), resultImage)
            Core.multiply(resultImage, Scalar(scale.toDouble(), scale.toDouble(), scale.toDouble(), 1.0), resultImage)

            resultImage
        } else {
            currentImage
        }

    }
}