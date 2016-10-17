package kr.edoli.imview.ui.view

import com.badlogic.gdx.Input
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
import com.badlogic.gdx.scenes.scene2d.ui.Image
import com.badlogic.gdx.scenes.scene2d.ui.Widget
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
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

    val selectDrawBox = Rectangle()
    val zoomDrawBox = Rectangle()

    val mousePotition = Vector2()

    // Colors
    val selectOverlayRectColor = Colors.selectOverlay.cpy().mul(1f, 1f, 1f, 0.2f)
    val selectOverlayBorderColor = Colors.selectOverlay.cpy()
    val zoomOverlayRectColor = Colors.zoomOverlay.cpy().mul(1f, 1f, 1f, 0.2f)
    val zoomOverlayBorderColor = Colors.zoomOverlay.cpy()
    val mouseCrossColor = Colors.mouseCross.cpy()


    private var image: Pixmap? = null
    private var imageRegion: TextureRegion? = null


    init {
        touchable = Touchable.enabled
        addListener(ImageViewerController(this, imageProperty, mousePotition))

        Context.mainImage.subscribe {
            if (it != null) {
                imageProperty.width = it.width.toFloat()
                imageProperty.height = it.height.toFloat()
                imageRegion = TextureRegion(Texture(it))
            } else {
                imageProperty.width = 0f
                imageProperty.height = 0f
            }
            image = it
        }

        Bus.subscribe(SelectionCopyMessage::class.java) {
            if (image != null) {
                Context.selectBox.once {
                    val selectBox = it

                    if (selectBox.width != 0f && selectBox.height != 0f) {
                        Clipboard.copy(image,
                                selectBox.x.toInt(),
                                ((image as Pixmap).height - selectBox.y.toInt() - selectBox.height.toInt()),
                                selectBox.width.toInt(),
                                selectBox.height.toInt())
                    } else {
                        Clipboard.copy(image)
                    }

                }
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

        // draw select box
        Context.selectBox.once {
            selectDrawBox.set(
                    it.x * imageProperty.scale + imageProperty.x,
                    it.y * imageProperty.scale + imageProperty.y,
                    it.width * imageProperty.scale,
                    it.height * imageProperty.scale)
        }
        batch.color = selectOverlayRectColor
        batch.drawRect(selectDrawBox)
        batch.color = selectOverlayBorderColor
        batch.drawRectBorder(selectDrawBox, 0.5f)

        // draw zoom box
        Context.zoomBox.once {
            zoomDrawBox.set(
                    it.x * imageProperty.scale + imageProperty.x,
                    it.y * imageProperty.scale + imageProperty.y,
                    it.width * imageProperty.scale,
                    it.height * imageProperty.scale)
        }
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

    fun selectAll() {
        Context.selectBox.update { it.set(0f, 0f, imageProperty.width, imageProperty.height) }
    }


    fun selectNone() {
        Context.selectBox.update { it.reset() }
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
            val imageViewer: ImageViewer,
            val imageProperty: ImageProperty,
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

        override fun keyDown(event: InputEvent, keycode: Int): Boolean {
            if (keycode == Input.Keys.A && UIUtils.ctrl()) {
                imageViewer.selectAll()
            }

            if (keycode == Input.Keys.C && UIUtils.ctrl()) {
                Bus.send(SelectionCopyMessage())
            }

            if (keycode == Input.Keys.ESCAPE) {
                imageViewer.selectNone()
            }

            return super.keyDown(event, keycode)
        }

        override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
            initX = x
            initY = y
            prevX = x
            prevY = y

            if (UIUtils.ctrl()) {
                mode = Mode.zoom
                Context.zoomBox.update {
                    it.set(
                        screenToPixelX(initX),
                        screenToPixelY(initY),
                        0f, 0f)
                }
            } else if (UIUtils.shift()) {
                mode = Mode.select
                Context.selectBox.update {
                    it.set(
                        screenToPixelX(initX).floor(),
                        screenToPixelY(initY).floor(),
                        0f, 0f)
                }
            } else {
                mode = Mode.move
            }

            if (event.stage != null) {
                event.stage.keyboardFocus = imageViewer
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
                    Context.zoomBox.update {
                        it.set(
                                screenToPixelX(initX),
                                screenToPixelY(initY),
                                screenToPixelX(x) - screenToPixelX(initX),
                                screenToPixelY(y) - screenToPixelY(initY))
                                .adjust()
                                .clamp(0f, 0f, x2, y2)
                    }
                }
                Mode.select -> {
                    Context.selectBox.update {
                        it.set(
                                screenToPixelX(initX),
                                screenToPixelY(initY),
                                (screenToPixelX(x) - screenToPixelX(initX)),
                                (screenToPixelY(y) - screenToPixelY(initY)))
                                .adjust()
                                .digitize()
                                .clamp(0f, 0f, x2, y2)
                    }
                }
            }

            cursorMoved(x, y)

            prevX = x
            prevY = y
        }

        override fun touchUp(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int) {
            when(mode) {
                Mode.zoom -> {
                    // Zoom
                    Context.zoomBox.once {
                        val widthScale = imageViewer.width / it.width
                        val heightScale = imageViewer.height / it.height
                        imageProperty.scale = Math.min(widthScale, heightScale)
                        imageProperty.x = (imageViewer.width - (it.x * 2 + it.width) * imageProperty.scale) / 2
                        imageProperty.y = (imageViewer.height - (it.y * 2 + it.height) * imageProperty.scale) / 2

                        logScale = Math.log(imageProperty.scale.toDouble()).toFloat()

                    }
                    Context.zoomBox.update { it.reset() }
                }
            }
        }

        override fun mouseMoved(event: InputEvent?, x: Float, y: Float): Boolean {
            cursorMoved(x, y)
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

        fun cursorMoved(x: Float, y: Float) {
            val pixelX = screenToPixelX(x).toInt()
            val pixelY = screenToPixelY(y).toInt()

            Context.cursorPosition.update {
                it.set(pixelX, pixelY)
            }

            Context.cursorRGB.update {
                val image = Context.mainImage.get()
                if (image != null) {
                    val pixel = ImageProc.getPixel(image, pixelX, image.height - pixelY)
                    it[0] = pixel[0]
                    it[1] = pixel[1]
                    it[2] = pixel[2]
                }
                it
            }

            mousePosition.x = x
            mousePosition.y = y
        }

    }
}