package kr.edoli.edolview.util

import org.opencv.core.Rect
import kotlin.math.max
import kotlin.math.min


/**
 * Created by daniel on 16. 9. 10.
 */

fun String.isInt(): Boolean = toIntOrNull() !== null
fun String.isFloat(): Boolean = toFloatOrNull() !== null

fun Int.clamp(min: Int, max: Int): Int = max(min, min(this, max))

fun Double.clamp(min: Double, max: Double): Double = max(min, min(this, max))
fun Double.floor(): Double = kotlin.math.floor(this)
fun Double.ceil(): Double = kotlin.math.ceil(this)

fun Float.clamp(min: Float, max: Float): Float = max(min, min(this, max))
fun Float.floor(): Float = kotlin.math.floor(this.toDouble()).toFloat()
fun Float.ceil(): Float = kotlin.math.ceil(this.toDouble()).toFloat()


fun Rect.hflip(targetHeight: Int): Rect {
    y = targetHeight - y - height
    return this
}

fun Rect.set(x: Int, y: Int, width: Int, height: Int): Rect {
    this.x = x
    this.y = y
    this.width = width
    this.height = height
    return this
}

fun Rect.reset(): Rect {
    x = 0
    y = 0
    width = 0
    height = 0
    return this
}

fun Rect.adjust(): Rect {
    if (width < 0) {
        width = -width
        x -= width
    }
    if (height < 0) {
        height = -height
        y -= height
    }
    return this
}

fun Rect.clamp(x1: Int, y1: Int, x2: Int, y2: Int): Rect {
    if (x + width < x1 || x > x2 || y + height < y1 || y > y2) {
        reset()
        return this
    }

    if (x < x1) {
        width -= (x1 - x)
        x = x1
    }
    if (y < y1) {
        height -= (y1 - y)
        y = y1
    }
    if (x + width > x2) width = x2 - x
    if (y + height > y2) height = y2 - y
    return this
}
