package kr.edoli.imview.util

/**
 * Created by daniel on 16. 9. 10.
 */

fun Double.clamp(min: Double, max: Double): Double = Math.max(min, Math.min(this, max))

fun Float.clamp(min: Float, max: Float): Float = Math.max(min, Math.min(this, max))