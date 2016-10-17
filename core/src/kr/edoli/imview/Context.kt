package kr.edoli.imview

import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.math.GridPoint2
import com.badlogic.gdx.math.Rectangle
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.ObservableValue

/**
 * Created by daniel on 16. 10. 2.
 */
object Context {
    val args = ObservableValue(arrayOf<String>())

    val mainImage = NullableObservableValue<Pixmap>(null)

    val cursorPosition = ObservableValue(GridPoint2())
    val cursorRGB = ObservableValue(byteArrayOf(0, 0, 0))
    val selectBox = ObservableValue(Rectangle())
    val zoomBox = ObservableValue(Rectangle())
}