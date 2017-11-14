package kr.edoli.imview

import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.math.GridPoint2
import com.badlogic.gdx.math.Rectangle
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.ObservableValue
import org.opencv.core.Mat
import org.opencv.core.Rect

/**
 * Created by daniel on 16. 10. 2.
 */
object Context {
    val args = ObservableValue(arrayOf<String>())

    val mainImage = NullableObservableValue<Mat>(null)
    val selectedImage = NullableObservableValue<Mat>(null)

    val mainPath = ObservableValue("")

    val cursorPosition = ObservableValue(GridPoint2())
    val cursorRGB = ObservableValue(intArrayOf())
    val selectBox = ObservableValue(Rect())
    val zoomBox = ObservableValue(Rect())
    val zoomRate = ObservableValue(1f)

    val comparisonMode = ObservableValue(ComparisonMode.Diff)
    val comparisonMetric = ObservableValue(ComparisonMetric.PSNR)

    val isShowCrosshair = ObservableValue(true)
    val isFixToolBar = ObservableValue(true)
    val isShowInfo = ObservableValue(false)
}