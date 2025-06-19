package kr.edoli.edolview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.PolygonBatch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.edolview.ui.*
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

open class RoundedBorderedDrawable(private val fill: Color, private val borderColor: Color? = null) : BaseDrawable() {

    var borderWidth = 1f
    var borderRadius = 0f
    var segments = 16
    var isBorder = true
    var topBorder = true
    var bottomBorder = true
    var leftBorder = true
    var rightBorder = true

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch as PolygonBatch

        // 내부 채우기
        batch.color = fill

        if (borderRadius > 0) {
            batch.drawRoundRect(x, y, width, height, borderRadius)
        } else {
            // 모서리가 둥글지 않은 경우 - 사각형 하나로 그림
            batch.drawQuad(x, y, x + width, y, x + width, y + height, x, y + height)
        }

        // 테두리 그리기
        if (isBorder && borderColor != null) {
            batch.color = borderColor

            if (borderRadius > 0) {
                // 둥근 모서리가 있는 경우의 테두리
                if (topBorder) {
                    // 위쪽 테두리
                    batch.drawLine(x + borderRadius, y + height, x + width - borderRadius, y + height, borderWidth)
                }
                if (rightBorder) {
                    // 오른쪽 테두리
                    batch.drawLine(x + width, y + borderRadius, x + width, y + height - borderRadius, borderWidth)
                }
                if (bottomBorder) {
                    // 아래쪽 테두리
                    batch.drawLine(x + borderRadius, y, x + width - borderRadius, y, borderWidth)
                }
                if (leftBorder) {
                    // 왼쪽 테두리
                    batch.drawLine(x, y + borderRadius, x, y + height - borderRadius, borderWidth)
                }

                // 둥근 모서리 부분의 테두리
                if (topBorder || rightBorder) {
                    // 오른쪽 위 모서리
                    batch.drawPath(roundedCornerBorder(x + width - borderRadius, y + height - borderRadius, borderRadius, 1), borderWidth)
                }
                if (rightBorder || bottomBorder) {
                    // 오른쪽 아래 모서리
                    batch.drawPath(roundedCornerBorder(x + width - borderRadius, y + borderRadius, borderRadius, 2), borderWidth)
                }
                if (bottomBorder || leftBorder) {
                    // 왼쪽 아래 모서리
                    batch.drawPath(roundedCornerBorder(x + borderRadius, y + borderRadius, borderRadius, 3), borderWidth)
                }
                if (leftBorder || topBorder) {
                    // 왼쪽 위 모서리
                    batch.drawPath(roundedCornerBorder(x + borderRadius, y + height - borderRadius, borderRadius, 0), borderWidth)
                }
            } else {
                // 직각 모서리가 있는 경우의 테두리
                if (topBorder) {
                    batch.drawLine(x, y + height, x + width, y + height, borderWidth)
                }
                if (rightBorder) {
                    batch.drawLine(x + width, y, x + width, y + height, borderWidth)
                }
                if (bottomBorder) {
                    batch.drawLine(x, y, x + width, y, borderWidth)
                }
                if (leftBorder) {
                    batch.drawLine(x, y, x, y + height, borderWidth)
                }
            }
        }
    }

    private fun roundedCornerBorder(x: Float, y: Float, radius: Float, corner: Int): FloatArray {
        val vertices = FloatArray((segments + 1) * 2)

        val startAngle = when (corner) {
            0 -> PI.toFloat() * 0.5f // 왼쪽 위
            1 -> 0f // 오른쪽 위
            2 -> PI.toFloat() * 1.5f // 오른쪽 아래
            3 -> PI.toFloat() // 왼쪽 아래
            else -> 0f
        }

        for (i in 0..segments) {
            val angle = startAngle + i * (PI.toFloat() * 0.5f) / segments
            val cosAngle = cos(angle)
            val sinAngle = sin(angle)

            vertices[i * 2] = x + cosAngle * radius
            vertices[i * 2 + 1] = y + sinAngle * radius
        }

        return vertices
    }

    fun pad(all: Float) {
        topHeight = all
        rightWidth = all
        bottomHeight = all
        leftWidth = all
    }

    fun pad(top: Float, right: Float, bottom: Float, left: Float) {
        topHeight = top
        rightWidth = right
        bottomHeight = bottom
        leftWidth = left
    }
}