package kr.edoli.edolview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.PolygonSpriteBatch
import com.badlogic.gdx.math.DelaunayTriangulator
import com.badlogic.gdx.math.MathUtils
import com.badlogic.gdx.math.Rectangle

private val vertices = FloatArray(2000)
private val indicies = ShortArray(2000)

private var idx = 0
private var index = 0


object Textures {
    val white = produceWhiteTexture()

    private fun produceWhiteTexture(): Texture {
        val pixmap = Pixmap(1, 1, Pixmap.Format.RGB565)

        pixmap.setColor(Color.WHITE)
        pixmap.fill()

        val tex = Texture(pixmap)
        tex.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)

        pixmap.dispose()

        return tex
    }
}

fun Batch.drawRect(rect: Rectangle) {
    drawRect(rect.x, rect.y, rect.width, rect.height)
}

fun Batch.drawRect(x: Float, y: Float, width: Float, height: Float) {
    drawQuad(x, y, x + width, y, x + width, y + height, x, y + height)
}

fun Batch.drawQuad(x1: Float, y1: Float, x2: Float, y2: Float,
                   x3: Float, y3: Float, x4: Float, y4: Float) {

    idx = 0
    quad(x1, y1, x2, y2, x3, y3, x4, y4)
    draw(Textures.white, vertices, 0, 20)
}

fun PolygonSpriteBatch.drawPolygon(points: FloatArray) {
    val triangles = DelaunayTriangulator().computeTriangles(points, true)
    idx = 0
    index = 0

    for (i in 0 until points.size / 2) {
        vertices[idx++] = points[i * 2]
        vertices[idx++] = points[i * 2 + 1]
        vertices[idx++] = packedColor
        vertices[idx++] = 0f
        vertices[idx++] = 0f
    }

    draw(Textures.white, vertices, 0, idx, triangles.toArray(), 0, triangles.size)
}

fun PolygonSpriteBatch.drawRoundRect(x: Float, y: Float, width: Float, height: Float, radius: Float) {
    idx = 0
    index = 0

    val x1 = x
    val y1 = y
    val x2 = x + width
    val y2 = y + height

    val padX1 = x1 + radius
    val padY1 = y1 + radius
    val padX2 = x2 - radius
    val padY2 = y2 - radius

    arc(padX2, padY1, radius, -90f, 0f)
    arc(padX2, padY2, radius, 0f, 90f)
    arc(padX1, padY2, radius, 90f, 180f)
    arc(padX1, padY1, radius, 180f, 270f)

    triangleFan(0, idx / 5)

    draw(Textures.white, vertices, 0, idx, indicies, 0, index)
}

fun PolygonSpriteBatch.drawCircle(x: Float, y: Float, radius: Float) {
    drawArc(x, y, radius, 0f, 360f)
}

fun PolygonSpriteBatch.drawArc(x: Float, y: Float, radius: Float, fromAngle: Float, toAngle: Float) {
    idx = 0
    index = 0

    vertices[idx++] = x
    vertices[idx++] = y
    vertices[idx++] = packedColor
    vertices[idx++] = 0f
    vertices[idx++] = 0f

    arc(x, y, radius, fromAngle, toAngle)

    triangleFan(0, idx / 5)

    draw(Textures.white, vertices, 0, idx, indicies, 0, index)
}

fun Batch.drawLine(x1: Float, y1: Float, x2: Float, y2: Float, lineWidth: Float) {
    val dx = x2 - x1
    val dy = y2 - y1

    val len = Math.hypot(dx.toDouble(), dy.toDouble()).toFloat()

    val halfWidth = lineWidth / 2
    val ndx = (dx / len) * halfWidth
    val ndy = (dy / len) * halfWidth

    drawQuad(x1 + ndy, y1 - ndx,
            x1 - ndy, y1 + ndx,
            x2 - ndy, y2 + ndx,
            x2 + ndy, y2 - ndx)
}

fun Batch.drawRectBorder(rect: Rectangle, lineWidth: Float) {
    drawRectBorder(rect.x, rect.y, rect.width, rect.height, lineWidth)
}

fun Batch.drawRectBorder(x: Float, y: Float, width: Float, height: Float, lineWidth: Float) {
    drawBorder(x, y, x + width, y, x + width, y + height, x, y + height, lineWidth)
}

fun Batch.drawBorder(x1: Float, y1: Float, x2: Float, y2: Float
                     , x3: Float, y3: Float, x4: Float, y4: Float, lineWidth: Float) {
    drawLine(x1 + 0.5f, y1 + 0.5f, x2 - 0.5f, y2 + 0.5f, lineWidth)
    drawLine(x2 - 0.5f, y2 + 0.5f, x3 - 0.5f, y3 - 0.5f, lineWidth)
    drawLine(x3 - 0.5f, y3 - 0.5f, x4 + 0.5f, y4 - 0.5f, lineWidth)
    drawLine(x4 + 0.5f, y4 - 0.5f, x1 + 0.5f, y1 + 0.5f, lineWidth)
}

private fun Batch.line(x1: Float, y1: Float, x2: Float, y2: Float) {
    val color = packedColor

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
}

private fun Batch.quad(x1: Float, y1: Float, x2: Float, y2: Float,
                       x3: Float, y3: Float, x4: Float, y4: Float) {

    val color = packedColor

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

}

private fun Batch.arc(x: Float, y: Float, radius: Float, fromAngle: Float, toAngle: Float) {
    val color = packedColor

    var angle = fromAngle
    var deltaAngle = (toAngle - fromAngle) / 15

    for (i in 0..15) {
        vertices[idx++] = x + radius * MathUtils.cosDeg(angle)
        vertices[idx++] = y + radius * MathUtils.sinDeg(angle)
        vertices[idx++] = color
        vertices[idx++] = 0f
        vertices[idx++] = 0f

        angle += deltaAngle
    }
}

private fun Batch.triangleFan(start: Int, count: Int) {
    for (i in 0..count-2) {
        indicies[index++] = start.toShort()
        indicies[index++] = (start + i).toShort()
        indicies[index++] = (start + i + 1).toShort()
    }
}
