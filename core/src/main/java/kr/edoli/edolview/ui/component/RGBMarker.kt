package kr.edoli.edolview.ui.component

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.glutils.ShapeRenderer
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.geom.Point2D
import kr.edoli.edolview.image.contains
import kr.edoli.edolview.image.get

class RGBMarker: Marker() {

    val tooltip = RGBTooltip()
    val shapeRenderer = ShapeRenderer()

    init {
        tooltip.setPosition(16f, 0f)
        addActor(tooltip)
    }

    override fun update() {
        val mainImage = ImContext.mainImage.get()
        val point = Point2D(imageX, imageY).cvPoint

        if (mainImage != null && mainImage.contains(point)) {
            val color = mainImage[point]
            tooltip.updateColor(color)
        } else {
            tooltip.clearColor()
        }
    }

    override fun drawChildren(batch: Batch, parentAlpha: Float) {
        batch.end()

        shapeRenderer.transformMatrix = batch.transformMatrix
        shapeRenderer.projectionMatrix = batch.projectionMatrix

        shapeRenderer.begin(ShapeRenderer.ShapeType.Filled)
        shapeRenderer.color = Color.DARK_GRAY
        shapeRenderer.circle(0.0f, 0.0f, 5.0f)
        shapeRenderer.color = Color.RED
        shapeRenderer.circle(0.0f, 0.0f, 3.0f)
        shapeRenderer.end()

        batch.begin()
        super.drawChildren(batch, parentAlpha)
    }
}