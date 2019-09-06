package kr.edoli.imview.util

fun Double.format(digits: Int) = java.lang.String.format("%.${digits}f", this)
fun Float.format(digits: Int) = java.lang.String.format("%.${digits}f", this)
