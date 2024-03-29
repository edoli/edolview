package kr.edoli.edolview.geom

import com.badlogic.gdx.math.Intersector
import com.badlogic.gdx.math.Vector2

object CustomIntersector {
    fun intersectSegmentRectangle(v1: Vector2, v2: Vector2, x: Float, y: Float, width: Float, height: Float): Vector2? {
        val r1 = Vector2()
        val r2 = Vector2()
        val intersection = Vector2()
        var isIntersected = false

        r1.set(x, y)
        r2.set(x + width, y)
        isIntersected = Intersector.intersectSegments(v1, v2, r1, r2, intersection)

        if (!isIntersected) {
            r1.set(r2)
            r2.set(x + width, y + height)
            isIntersected = Intersector.intersectSegments(v1, v2, r1, r2, intersection)
        }
        if (!isIntersected) {
            r1.set(r2)
            r2.set(x, y + height)
            isIntersected = Intersector.intersectSegments(v1, v2, r1, r2, intersection)
        }
        if (!isIntersected) {
            r1.set(r2)
            r2.set(x, y)
            isIntersected = Intersector.intersectSegments(v1, v2, r1, r2, intersection)
        }

        return if (isIntersected) {
            intersection
        } else {
            null
        }
    }
}