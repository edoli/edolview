package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.ui.Widget

/**
 * Created by daniel on 16. 9. 23.
 */
class ColorWidget : Widget {

    constructor(color: Color) : super() {
        this.color = color
    }

    override fun draw(batch: Batch, parentAlpha: Float) {
        super.draw(batch, parentAlpha)

        val c = batch.color

        batch.color = color

        batch.drawRect(x, y, width, height)

        batch.color = c
    }
}