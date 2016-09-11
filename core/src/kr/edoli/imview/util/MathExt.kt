package kr.edoli.imview.util

import com.badlogic.gdx.math.Rectangle


/**
 * Created by daniel on 16. 9. 10.
 */

fun Double.clamp(min: Double, max: Double): Double = Math.max(min, Math.min(this, max))
fun Double.floor(): Double = Math.floor(this)
fun Double.ceil(): Double = Math.ceil(this)

fun Float.clamp(min: Float, max: Float): Float = Math.max(min, Math.min(this, max))
fun Float.floor(): Float = Math.floor(this.toDouble()).toFloat()
fun Float.ceil(): Float = Math.ceil(this.toDouble()).toFloat()

fun Rectangle.adjust(): Rectangle {
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
fun Rectangle.digitize(): Rectangle {
    val px = x
    val py = y
    x = x.floor()
    y = y.floor()
    width = (px - x + width).ceil()
    height = (py - y + height).ceil()
    return this
}
fun Rectangle.reset(): Rectangle {
    x = 0f
    y = 0f
    width = 0f
    height = 0f
    return this
}