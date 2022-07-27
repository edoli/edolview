package kr.edoli.imview.util

import com.badlogic.gdx.graphics.Color
import kr.edoli.imview.image.ImageSpec


fun DoubleArray.toColor(): Color {
    val length = this.size
    return if (length == 4 || length == 3) {
        Color(this[0].toFloat(), this[1].toFloat(), this[2].toFloat(), 1f)
    } else if (length == 1) {
        Color(this[0].toFloat(), this[0].toFloat(), this[0].toFloat(), 1f)
    } else {
        Color.BLACK
    }
}

fun DoubleArray.toColorStr(imageSpec: ImageSpec) = map {
    // scale == 0 means that an opened image is not uint image
    val scale = imageSpec.typeMaxValue

    var value = it
    if (scale > 0) {
        value *= scale
    }
    if (imageSpec.isInt) value.format(0) else value
}.joinToString(", ")

fun Color.toFloatArray() = floatArrayOf(this.r, this.g, this.b, this.a)