package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.math.Rectangle
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Widget
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.CursorPositionMessage
import kr.edoli.imview.bus.SelectBoxMessage
import kr.edoli.imview.bus.SelectionCopyMessage
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.drawLine
import kr.edoli.imview.ui.drawRect
import kr.edoli.imview.ui.drawRectBorder
import kr.edoli.imview.util.*

/**
 * Created by daniel on 16. 9. 10.
 */
class ImageViewer : Widget() {

    val viewerId = generateId()
    val imageProperty = ImageProperty()

    val selectBox = Rectangle()
    val zoomBox = Rectangle()

    val selectDrawBox = Rectangle()
    val zoomDrawBox = Rectangle()

    val mousePotition = Vector2()

    // Colors
    val selectOverlayRectColor = Colors.selectOverlay.cpy().mul(1f, 1f, 1f, 0.2f)
    val selectOverlayBorderColor = Colors.selectOverlay.cpy()
    val zoomOverlayRectColor = Colors.zoomOverlay.cpy().mul(1f, 1f, 1f, 0.2f)
    val zoomOverlayBorderColor = Colors.zoomOverlay.cpy()
    val mouseCrossColor = Colors.mouseCross.cpy()


    var image: Pixmap? = null
        set(value) {
            if (value != null) {
                imageProperty.width = value.width.toFloat()
                imageProperty.height = value.height.toFloat()
                imageRegion = TextureRegion(Texture(value))
            } else {
                imageProperty.width = 0f
                imageProperty.height = 0f
            }
            field = value
        }
    var imageRegion: TextureRegion? = null


    init {
        touchable = Touchable.enabled
        addListener(ImageViewerController(this, imageProperty, zoomBox, selectBox, mousePotition))

        Bus.subscribe(SelectionCopyMessage::class.java) {
            if (image != null) {
                Clipboard.copy(image,
                        selectBox.x.toInt(),
                        ((image as Pixmap).height - selectBox.y.toInt() - selectBox.height.toInt()),
                        selectBox.width.toInt(),
                        selectBox.height.toInt())
            }
        }
    }

    override fun hit(x: Float, y: Float, touchable: Boolean): Actor? {
        return super.hit(x, y, touchable)
    }

    override fun draw(batch: Batch, parentAlpha: Float) {
        // draw image
        if (image != null) {
            batch.draw(imageRegion, imageProperty.x, imageProperty.y,
                    0f, 0f,
                    imageProperty.width, imageProperty.height,
                    imageProperty.scale, imageProperty.scale, 0f)
        }

        val prevColor = batch.color

        // draw overlay
        selectDrawBox.set(
                selectBox.x * imageProperty.scale + imageProperty.x,
                selectBox.y * imageProperty.scale + imageProperty.y,
                selectBox.width * imageProperty.scale,
                selectBox.height * imageProperty.scale)
        batch.color = selectOverlayRectColor
        batch.drawRect(selectDrawBox)
        batch.color = selectOverlayBorderColor
        batch.drawRectBorder(selectDrawBox, 0.5f)

        zoomDrawBox.set(
                zoomBox.x * imageProperty.scale + imageProperty.x,
                zoomBox.y * imageProperty.scale + imageProperty.y,
                zoomBox.width * imageProperty.scale,
                zoomBox.height * imageProperty.scale)
        batch.color = zoomOverlayRectColor
        batch.drawRect(zoomDrawBox)
        batch.color = zoomOverlayBorderColor
        batch.drawRectBorder(zoomDrawBox, 0.5f)

        // draw mouse cross
        batch.color = mouseCrossColor
        batch.drawLine(0f, mousePotition.y + 0.5f, width, mousePotition.y + 0.5f, 1f)
        batch.drawLine(mousePotition.x + 0.5f, 0f, mousePotition.x + 0.5f, height, 1f)


        batch.color = prevColor

        super.draw(batch, parentAlpha)
    }

    data class ImageProperty(
            var width: Float = 0f,
            var height: Float = 0f,
            var x: Float = 0f,
            var y: Float = 0f,
            var scale: Float = 1f
    )


    // Controller
    class ImageViewerController(
            val imageViewer: Actor,
            val imageProperty: ImageProperty,
            val zoomBox: Rectangle,
            val selectBox: Rectangle,
            val mousePosition: Vector2) : InputListener() {

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

            val x2 = imageProperty.width
            val y2 = imageProperty.height

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
                            .clamp(0f, 0f, x2, y2)
                }
                Mode.select -> {
                    selectBox.set(
                            screenToPixelX(initX),
                            screenToPixelY(initY),
                            (screenToPixelX(x) - screenToPixelX(initX)),
                            (screenToPixelY(y) - screenToPixelY(initY)))
                            .adjust()
                            .digitize()
                            .clamp(0f, 0f, x2, y2)


                    Bus.send(SelectBoxMessage(
                            selectBox.x.toInt(),
                            selectBox.y.toInt(),
                            selectBox.width.toInt(),
                            selectBox.height.toInt()))
                }
            }

            moved(x, y)

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

        override fun mouseMoved(event: InputEvent?, x: Float, y: Float): Boolean {
            moved(x, y)
            return true
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

        fun moved(x: Float, y: Float) {
            Bus.send(CursorPositionMessage(
                    screenToPixelX(x).toInt(),
                    screenToPixelY(y).toInt()))

            mousePosition.x = x
            mousePosition.y = y
        }

    }
}