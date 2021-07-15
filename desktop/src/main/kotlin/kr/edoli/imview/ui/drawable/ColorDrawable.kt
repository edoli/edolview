package kr.edoli.imview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.imview.ui.UIRes


class ColorDrawable(val color: Color) : BaseDrawable() {

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = color
        batch.draw(UIRes.white, x, y, width, height)
    }
}