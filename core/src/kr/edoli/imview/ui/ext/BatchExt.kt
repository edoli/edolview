package kr.edoli.imview.ui.ext

import com.badlogic.gdx.graphics.g2d.Batch
import kr.edoli.imview.ui.White

/**
 * Created by daniel on 16. 9. 10.
 */
private val vertices = FloatArray(20)


fun Batch.drawRect(x: Float, y: Float, width: Float, height: Float) {
    drawQuad(x, y, x + width, y, x + width, y + height, x, y + height);
}

fun Batch.drawQuad(x1: Float, y1: Float, x2: Float, y2: Float,
                   x3: Float, y3: Float, x4: Float, y4: Float) {

    val color = packedColor

    var idx = 0

    vertices[idx++] = x1
    vertices[idx++] = y1
    vertices[idx++] = color
    vertices[idx++] = 0f
    vertices[idx++] = 0f

    vertices[idx++] = x2
    vertices[idx++] = y2
    vertices[idx++] = color
    vertices[idx++] = 1f
    vertices[idx++] = 0f

    vertices[idx++] = x3
    vertices[idx++] = y3
    vertices[idx++] = color
    vertices[idx++] = 0f
    vertices[idx++] = 1f

    vertices[idx++] = x4
    vertices[idx++] = y4
    vertices[idx++] = color
    vertices[idx++] = 1f
    vertices[idx++] = 1f

    draw(White, vertices, 0, 20);
}

fun Batch.drawLine(x1: Float, y1: Float, x2: Float, y2: Float, lineWidth: Float) {
    val dx = x2 - x1
    val dy = y2 - y1

    val len = Math.hypot(dx.toDouble(), dy.toDouble()).toFloat()

    val halfWidth = lineWidth / 2
    val ndx = (dx / len) * lineWidth
    val ndy = (dy / len) * lineWidth

    drawQuad(x1 + ndy, y1 - ndx,
             x1 - ndy, y1 + ndx,
             x2 - ndy, y2 + ndx,
             x2 + ndy, y2 - ndx)
}

fun Batch.drawRectBorder(x: Float, y: Float, width: Float, height: Float, lineWidth: Float) {
    drawBorder(x, y, x + width, y, x + width, y + height, x, y + height, lineWidth);
}

fun Batch.drawBorder(x1: Float, y1: Float, x2: Float, y2: Float
, x3: Float, y3: Float, x4: Float, y4: Float, lineWidth: Float) {
    drawLine(x1, y1, x2, y2, lineWidth);
    drawLine(x2, y2, x3, y3, lineWidth);
    drawLine(x3, y3, x4, y4, lineWidth);
    drawLine(x4, y4, x1, y1, lineWidth);
}
