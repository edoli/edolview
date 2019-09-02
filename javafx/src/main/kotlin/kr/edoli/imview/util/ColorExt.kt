package kr.edoli.imview.util

import javafx.scene.paint.Color

fun DoubleArray.toColor(): Color {
    val length = this.size
    return if (length == 4 || length == 3) {
        val c = this.map { (it * 255).clamp(0.0, 255.0).toInt() }
        Color.rgb(c[0], c[1], c[2])
    } else if (length == 1) {
        Color.grayRgb(Math.min(this[0] * 255, 255.0).toInt())
    } else {
        Color.BLACK
    }
}

fun DoubleArray.toColorStr(maxValue: Double) = map {
    if (maxValue > 0) (it * maxValue).toInt().toString() else it.format(2)
}.joinToString(", ")