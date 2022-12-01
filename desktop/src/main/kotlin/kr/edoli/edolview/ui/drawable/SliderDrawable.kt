package kr.edoli.edolview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.edolview.ui.UIRes
import kr.edoli.edolview.ui.res.Colors
import kotlin.math.abs
import kotlin.math.min

open class SliderDrawable(private val fill: Color) : BaseDrawable() {

    var padding = 4f

    var drawDelta = false
    var defaultFraction = 0f
    var currentFraction = 0f

    init {
        minHeight = 4f
    }

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = fill
        batch.draw(UIRes.white, x + padding, y, width - padding * 2, height)

        if (drawDelta) {
            val deltaX = min(defaultFraction, currentFraction) * width
            val deltaWidth = abs(defaultFraction - currentFraction) * width

            if (currentFraction > defaultFraction) {
                batch.color = Colors.accent
            } else {
                batch.color = Colors.negative
            }

            batch.draw(UIRes.white, x + deltaX, y, deltaWidth, height)
        }
    }
}