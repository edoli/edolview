package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.math.Rectangle
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Touchable
import kr.edoli.imview.ui.ext.drawLine
import kr.edoli.imview.ui.ext.drawRectBorder
import kr.edoli.imview.util.generateId

/**
 * Created by daniel on 16. 9. 10.
 */
class ImageViewer : Actor() {

    val viewerId = generateId()
    val imageProperty = ImageProperty()

    val selectBox = Rectangle()
    val zoomBox = Rectangle()

    var image: TextureRegion? = null
        set(value) {
            if (value != null) {
                imageProperty.width = value.regionWidth.toFloat()
                imageProperty.height = value.regionHeight.toFloat()
            } else {
                imageProperty.width = 0f
                imageProperty.height = 0f
            }
            field = value
        }


    init {
        touchable = Touchable.enabled
        addListener(ImageViewerController(imageProperty))
    }

    override fun draw(batch: Batch, parentAlpha: Float) {
        // draw image
        if (image != null) {
            batch.draw(image, imageProperty.x, imageProperty.y,
                    0f, 0f,
                    imageProperty.width, imageProperty.height,
                    imageProperty.scale, imageProperty.scale, 0f)
        }

        // draw overlay
        color = batch.color.cpy()
        batch.color.set(Color.RED)
        batch.drawRectBorder(selectBox, 1f)

        batch.color.set(color)


        super.draw(batch, parentAlpha)
    }

    data class ImageProperty(
            var width: Float = 0f,
            var height: Float = 0f,
            var x: Float = 0f,
            var y: Float = 0f,
            var scale: Float = 1f
    )

    class ImageViewerController(val imageProperty: ImageProperty) : InputListener() {

        var prevX = 0f
        var prevY = 0f
        var logScale = 0f

        init {
            logScale = Math.log(imageProperty.scale.toDouble()).toFloat()
        }

        override fun touchDown(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int): Boolean {
            prevX = x
            prevY = y

            return true
        }

        override fun touchDragged(event: InputEvent?, x: Float, y: Float, pointer: Int) {
            val dx = x - prevX
            val dy = y - prevY

            imageProperty.x += dx
            imageProperty.y += dy

            prevX = x
            prevY = y
        }

        override fun touchUp(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int) {
        }

        override fun enter(event: InputEvent?, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
            if (event != null) {
                event.stage.scrollFocus = event.target
            }
        }

        override fun scrolled(event: InputEvent?, x: Float, y: Float, amount: Int): Boolean {
            val fracX = (x - imageProperty.x) / imageProperty.scale
            val fracY = (y - imageProperty.y) / imageProperty.scale

            logScale += amount / 16f
            imageProperty.scale = Math.exp(logScale.toDouble()).toFloat()

            imageProperty.x = x - fracX * imageProperty.scale
            imageProperty.y = y - fracY * imageProperty.scale

            return true
        }

    }
}