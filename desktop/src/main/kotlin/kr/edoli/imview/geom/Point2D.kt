package kr.edoli.imview.geom

import org.opencv.core.Point

class Point2D(
        val x: Double,
        val y: Double
) {
    val cvPoint: Point
        get() = Point(x, y)

    override fun toString(): String {
        return "($x, $y)"
    }
}