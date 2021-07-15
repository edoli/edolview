package kr.edoli.imview.ui.drawable

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.utils.BaseDrawable
import kr.edoli.imview.ui.UIRes

open class BorderedDrawable(val fill: Color, val border: Color) : BaseDrawable() {

    var borderWidth = 1f
    var isBorder = true
    var topBorder = true
    var bottomBorder = true
    var leftBorder = true
    var rightBorder = true

    override fun draw(batch: Batch, x: Float, y: Float, width: Float, height: Float) {
        batch.color = fill
        batch.draw(UIRes.white, x, y, width, height)

        if (isBorder) {
            batch.color = border
            if (topBorder) {
                batch.draw(UIRes.white, x, y, width, borderWidth)
            }
            if (leftBorder) {
                batch.draw(UIRes.white, x, y, borderWidth, height)
            }
            if (bottomBorder) {
                batch.draw(UIRes.white, x, y + height - borderWidth, width, borderWidth)
            }
            if (rightBorder) {
                batch.draw(UIRes.white, x + width - borderWidth, y, borderWidth, height)
            }
        }
    }
}