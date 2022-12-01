package kr.edoli.edolview.ui.custom

import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.ui.Skin
import com.badlogic.gdx.scenes.scene2d.ui.Slider
import kr.edoli.edolview.ui.drawable.SliderDrawable

class CustomSlider(min: Float, max: Float, stepSize: Float, vertical: Boolean, skin: Skin)
    : Slider(min, max, stepSize, vertical, skin) {

    var defaultValue = 0f
        set(value) {
            hasDefaultValue = true
            field = value
        }
    var hasDefaultValue = false

    override fun getPrefWidth(): Float {
        val knob = style.knob
        val bg = backgroundDrawable
        return (knob?.minWidth ?: 0f) + (bg?.minWidth ?: 0f)
    }

    override fun draw(batch: Batch?, parentAlpha: Float) {
        val bg = backgroundDrawable
        if (bg != null && bg is SliderDrawable && hasDefaultValue) {
            bg.drawDelta = hasDefaultValue
            bg.currentFraction = percent
            bg.defaultFraction = if (minValue == maxValue) 0f else (defaultValue - minValue) / (maxValue - minValue)
        }
        super.draw(batch, parentAlpha)
    }
}