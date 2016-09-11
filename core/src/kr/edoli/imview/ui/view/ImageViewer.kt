package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.math.Rectangle
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import kr.edoli.imview.ui.ext.drawLine
import kr.edoli.imview.ui.ext.drawRect
import kr.edoli.imview.ui.ext.drawRectBorder
import kr.edoli.imview.util.*

/**
 * Created by daniel on 16. 9. 10.
 */
class ImageViewer : Actor() {

    val viewerId = generateId()
    val imageProperty = ImageProperty()

    val selectBox = Rectangle()
    val zoomBox = Rectangle()

    val selectDrawBox = Rectangle()
    val zoomDrawBox = Rectangle()

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
        addListener(ImageViewerController(this, imageProperty, zoomBox, selectBox))
    }

    override fun hit(x: Float, y: Float, touchable: Boolean): Actor? {
        return super.hit(x, y, touchable)
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
        color = batch.color

        selectDrawBox.set(
                selectBox.x * imageProperty.scale + imageProperty.x,
                selectBox.y * imageProperty.scale + imageProperty.y,
                selectBox.width * imageProperty.scale,
                selectBox.height * imageProperty.scale)
        batch.color = Color(1f, 1f, 1f, 0.2f)
        batch.drawRect(selectDrawBox)
        batch.color = Color.GREEN
        batch.drawRectBorder(selectDrawBox, 1f)

        zoomDrawBox.set(
                zoomBox.x * imageProperty.scale + imageProperty.x,
                zoomBox.y * imageProperty.scale + imageProperty.y,
                zoomBox.width * imageProperty.scale,
                zoomBox.height * imageProperty.scale)
        batch.color = Color(1f, 1f, 1f, 0.2f)
        batch.drawRect(zoomDrawBox)
        batch.color = Color.RED
        batch.drawRectBorder(zoomDrawBox, 1f)

        batch.color = color

        super.draw(batch, parentAlpha)
    }

    data class ImageProperty(
            var width: Float = 0f,
            var height: Float = 0f,
            var x: Float = 0f,
            var y: Float = 0f,
            var scale: Float = 1f
    )

    class ImageViewerController(
            val imageViewer: Actor,
            val imageProperty: ImageProperty,
            val zoomBox: Rectangle,
            val selectBox: Rectangle) : InputListener() {

        enum class Mode {
            move, select, zoom
        }

        var mode = Mode.move
        var initX = 0f
        var initY = 0f
        var prevX = 0f
        var prevY = 0f
        var logScale = 0f

        init {
            logScale = Math.log(imageProperty.scale.toDouble()).toFloat()
        }

        fun screenToPixelX(x: Float): Float {
            return (x - imageProperty.x) / imageProperty.scale
        }

        fun screenToPixelY(y: Float): Float {
            return (y - imageProperty.y) / imageProperty.scale
        }

        override fun touchDown(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int): Boolean {
            initX = x
            initY = y
            prevX = x
            prevY = y

            if (UIUtils.ctrl()) {
                mode = Mode.zoom
                zoomBox.set(
                        screenToPixelX(initX),
                        screenToPixelY(initY),
                        0f, 0f)
            } else if (UIUtils.shift()) {
                mode = Mode.select
                selectBox.set(
                        screenToPixelX(initX).floor(),
                        screenToPixelY(initY).floor(),
                        0f, 0f)
            } else {
                mode = Mode.move
            }

            return true
        }

        override fun touchDragged(event: InputEvent?, x: Float, y: Float, pointer: Int) {
            val dx = x - prevX
            val dy = y - prevY

            when(mode) {
                Mode.move -> {
                    imageProperty.x += dx
                    imageProperty.y += dy
                }
                Mode.zoom -> {
                    zoomBox.set(
                            screenToPixelX(initX),
                            screenToPixelY(initY),
                            screenToPixelX(x) - screenToPixelX(initX),
                            screenToPixelY(y) - screenToPixelY(initY))
                            .adjust()
                }
                Mode.select -> {
                    selectBox.set(
                            screenToPixelX(initX),
                            screenToPixelY(initY),
                            (screenToPixelX(x) - screenToPixelX(initX)),
                            (screenToPixelY(y) - screenToPixelY(initY)))
                            .adjust()
                            .digitize()
                }

            }

            prevX = x
            prevY = y
        }

        override fun touchUp(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int) {
            when(mode) {
                Mode.zoom -> {
                    val widthScale = imageViewer.width / zoomBox.width
                    val heightScale = imageViewer.height / zoomBox.height
                    imageProperty.scale = Math.min(widthScale, heightScale)
                    imageProperty.x = (imageViewer.width - (zoomBox.x * 2 + zoomBox.width) * imageProperty.scale) / 2
                    imageProperty.y = (imageViewer.height - (zoomBox.y * 2 + zoomBox.height) * imageProperty.scale) / 2

                    logScale = Math.log(imageProperty.scale.toDouble()).toFloat()

                    zoomBox.reset()
                }
            }
        }

        override fun enter(event: InputEvent?, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
            if (event != null) {
                event.stage.scrollFocus = event.target
            }
        }

        override fun scrolled(event: InputEvent?, x: Float, y: Float, amount: Int): Boolean {
            val fracX = (x - imageProperty.x) / imageProperty.scale
            val fracY = (y - imageProperty.y) / imageProperty.scale

            logScale -= amount / 16f
            imageProperty.scale = Math.exp(logScale.toDouble()).toFloat()

            imageProperty.x = x - fracX * imageProperty.scale
            imageProperty.y = y - fracY * imageProperty.scale

            return true
        }

    }
}