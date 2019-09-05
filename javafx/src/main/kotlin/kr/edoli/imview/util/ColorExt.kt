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

fun DoubleArray.toColorStr(scale: Double) = map {
    if (scale > 0) (it * scale).toInt().toString() else it.format(2)
}.joinToString(", ")