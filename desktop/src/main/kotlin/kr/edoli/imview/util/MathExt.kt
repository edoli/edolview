package kr.edoli.imview.util

import org.opencv.core.Rect


/**
 * Created by daniel on 16. 9. 10.
 */

fun Double.clamp(min: Double, max: Double): Double = Math.max(min, Math.min(this, max))
fun Double.floor(): Double = Math.floor(this)
fun Double.ceil(): Double = Math.ceil(this)

fun Float.clamp(min: Float, max: Float): Float = Math.max(min, Math.min(this, max))
fun Float.floor(): Float = Math.floor(this.toDouble()).toFloat()
fun Float.ceil(): Float = Math.ceil(this.toDouble()).toFloat()


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