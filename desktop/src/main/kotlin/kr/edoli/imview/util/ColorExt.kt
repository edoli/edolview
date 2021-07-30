package kr.edoli.imview.util

import com.badlogic.gdx.graphics.Color


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

fun DoubleArray.toColorStr(scale: Double, fixed: Int = -1) = map {
    // scale == 0 means that an opened image is not uint image
    if (scale > 0) (it * scale).toInt().toString() else (if (fixed == -1) it else it.format(fixed))
}.joinToString(", ")

fun Color.toFloatArray() = floatArrayOf(this.r, this.g, this.b, this.a)