package kr.edoli.edolview.util

import java.awt.GraphicsEnvironment
import java.util.*


fun generateId(): Long {
    return UUID.randomUUID().mostSignificantBits and java.lang.Long.MAX_VALUE
}

fun <T> functionTime(description: String = "", func: () -> T): T {
    val startTime = System.nanoTime()
    val value = func()
    println("${(System.nanoTime() - startTime).toFloat() / 1000 / 1000}ms [${description}]")
    return value
}

fun getScalingFactor(): Float {
    val ge = GraphicsEnvironment.getLocalGraphicsEnvironment().defaultScreenDevice.defaultConfiguration
    val transform = ge.defaultTransform
    return transform.scaleX.toFloat()
}