package kr.edoli.edoliui.widget.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.imview.ui.drawRect
import kr.edoli.imview.ui.drawRoundRect

/**
 * Created by daniel on 16. 6. 5.
 */
class ColorDrawable(
        private val color: Color,
        private val round: Float = 0f) : BaseDrawable() {

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        val preColor = batch.color

        batch.color = color

        if (round == 0f) {
            batch.drawRect(x, y, width, height)
        } else {
            batch.drawRoundRect(x, y, width, height, round)
        }

        batch.color = preColor

        super.draw(batch, x, y, width, height)
    }
}
