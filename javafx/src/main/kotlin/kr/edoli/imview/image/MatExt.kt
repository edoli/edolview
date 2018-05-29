package kr.edoli.imview.image

import javafx.geometry.Point2D
import javafx.scene.shape.Rectangle
import kr.edoli.imview.util.set
import org.opencv.core.*

operator fun Mat.get(rowRange: IntRange, colRange: IntRange): Mat {
    return submat(rowRange.cv, colRange.cv)
}

operator fun Mat.get(rect: Rect): Mat {
    return submat(rect)
}

operator fun Mat.get(row: Int, col: Int, channel: Int): Double {
    return get(row, col)[channel]
}

operator fun Mat.get(point: Point): DoubleArray {
    return get(point.y.toInt(), point.x.toInt())
}

fun Mat.contains(rect: Rect): Boolean {
    return rect.x >= 0 && rect.y >= 0 &&
            rect.x + rect.width <= this.width() &&
            rect.y + rect.height <= this.height()
}

fun Mat.contains(point: Point): Boolean {
    return point.x >= 0 && point.y >= 0 &&
            point.x < this.width() && point.y < this.height()
}

/* Plus */

operator fun Mat.plus(other: Mat): Mat {
    // check dimension and type of two matrices
    if (this.type() != other.type()) {
        throw Exception()
    }
    if (this.rows() != other.rows() || this.cols() != other.cols()) {
        throw Exception()
    }

    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.add(this, other, newMat)
    return newMat
}

operator fun Mat.plus(value: Double): Mat {
    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.add(this, Scalar(DoubleArray(newMat.channels()) { value }), newMat)
    return newMat
}

operator fun Double.plus(mat: Mat): Mat {
    val newMat = Mat(mat.rows(), mat.cols(), mat.type())
    Core.add(mat, Scalar(DoubleArray(newMat.channels()) { this }), newMat)
    return newMat
}

/* Minus */

operator fun Mat.minus(other: Mat): Mat {
    // check dimension and type of two matrices
    if (this.type() != other.type()) {
        throw Exception()
    }
    if (this.rows() != other.rows() || this.cols() != other.cols()) {
        throw Exception()
    }

    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.subtract(this, other, newMat)
    return newMat
}

operator fun Mat.minus(value: Double): Mat {
    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.subtract(this, Scalar(DoubleArray(newMat.channels()) { value }), newMat)
    return newMat
}

operator fun Double.minus(mat: Mat): Mat {
    val newMat = Mat(mat.rows(), mat.cols(), mat.type(), Scalar(DoubleArray(mat.channels()) { this }))
    Core.subtract(newMat, mat, newMat)
    return newMat
}

operator fun Mat.unaryMinus(): Mat {
    val newMat = Mat(this.rows(), this.cols(), this.type(), Scalar(DoubleArray(channels()) { 0.0 }))
    Core.subtract(newMat, this, newMat)
    return newMat
}

/* Times */

operator fun Mat.times(other: Mat): Mat {
    // check dimension and type of two matrices
    if (this.type() != other.type()) {
        throw Exception()
    }
    if (this.rows() != other.rows() || this.cols() != other.cols()) {
        throw Exception()
    }

    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.multiply(this, other, newMat)
    return newMat
}

operator fun Mat.times(value: Double): Mat {
    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.multiply(this, Scalar(DoubleArray(newMat.channels()) { value }), newMat)
    return newMat
}

operator fun Double.times(mat: Mat): Mat {
    val newMat = Mat(mat.rows(), mat.cols(), mat.type())
    Core.multiply(mat, Scalar(DoubleArray(newMat.channels()) { this }), newMat)
    return newMat
}

/* Divide */

operator fun Mat.div(other: Mat): Mat {
    // check dimension and type of two matrices
    if (this.type() != other.type()) {
        throw Exception()
    }
    if (this.rows() != other.rows() || this.cols() != other.cols()) {
        throw Exception()
    }

    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.divide(this, other, newMat)
    return newMat
}

operator fun Mat.div(value: Double): Mat {
    val newMat = Mat(this.rows(), this.cols(), this.type())
    Core.divide(this, Scalar(DoubleArray(newMat.channels()) { value }), newMat)
    return newMat
}

operator fun Double.div(mat: Mat): Mat {
    val newMat = Mat(mat.rows(), mat.cols(), mat.type(), Scalar(DoubleArray(mat.channels()) { this }))
    Core.divide(newMat, mat, newMat)
    return newMat
}

/* Type conversion */

val IntRange.cv: Range
    get() = Range(this.first, this.last)

val Rectangle.cv: Rect
    get() = Rect(x.toInt(), y.toInt(), width.toInt(), height.toInt())

val Point2D.cv: Point
    get() = Point(x, y)

fun Rect.set(rectangle: Rectangle): Rect {
    return this.set(rectangle.x.toInt(), rectangle.y.toInt(), rectangle.width.toInt(), rectangle.height.toInt())
}

fun Rect.set(rect: Rect): Rect {
    return this.set(rect.x, rect.y, rect.width, rect.height)
}

