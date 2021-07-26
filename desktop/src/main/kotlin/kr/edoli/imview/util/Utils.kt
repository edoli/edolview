package kr.edoli.imview.util

import java.io.File
import java.net.URL
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