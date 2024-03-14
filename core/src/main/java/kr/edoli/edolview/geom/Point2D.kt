package kr.edoli.edolview.geom

import org.opencv.core.Point

class Point2D(
        val x: Double,
        val y: Double
) {

    constructor(x: Float, y: Float) : this(x.toDouble(), y.toDouble())

    val cvPoint: Point
        get() = Point(x, y)

    override fun toString(): String {
        return "($x, $y)"
    }
}