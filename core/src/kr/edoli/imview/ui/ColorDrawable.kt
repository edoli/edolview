package kr.edoli.edoliui.widget.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.edoliui.widget.Textures

/**
 * Created by daniel on 16. 6. 5.
 */
class ColorDrawable(private val color: Color) : BaseDrawable() {

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        val preColor = batch.color

        batch.color = color
        batch.draw(Textures.white, x, y, width, height)

        batch.color = preColor

        super.draw(batch, x, y, width, height)
    }
}
