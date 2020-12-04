package kr.edoli.imview.ui.custom

import com.badlogic.gdx.scenes.scene2d.ui.Skin
import com.badlogic.gdx.scenes.scene2d.ui.Slider

class CustomSlider(min: Float, max: Float, stepSize: Float, vertical: Boolean, skin: Skin)
    : Slider(min, max, stepSize, vertical, skin) {

    override fun getPrefWidth(): Float {
        val knob = style.knob
        val bg = backgroundDrawable
        return (knob?.minWidth ?: 0f) + (bg?.minWidth ?: 0f)
    }
}