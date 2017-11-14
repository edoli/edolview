package kr.edoli.imview.ui.view

import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
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
import kr.edoli.imview.ComparisonMode
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.SelectionCopyMessage
import kr.edoli.imview.image.ImageProc
import kr.edoli.imview.image.ImageUtils
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.drawLine
import kr.edoli.imview.ui.drawRect
import kr.edoli.imview.ui.drawRectBorder
import kr.edoli.imview.util.*
import org.opencv.core.Mat
import org.opencv.core.Rect

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


    private var image: Mat? = null
    private var imageRegion: TextureRegion? = null
    private var overlayRegion: TextureRegion = TextureRegion()


    init {
        touchable = Touchable.enabled
        addListener(ImageViewerController(this, imageProperty, mousePotition))

        Context.mainImage.subscribe {
            if (it != null) {
                imageProperty.width = it.cols().toFloat()
                imageProperty.height = it.rows().toFloat()

                if (imageRegion != null) {
                    imageRegion?.texture?.dispose()
                }

                val pixmap = ImageUtils.matToPixmap(it)
                imageRegion = TextureRegion(Texture(pixmap))
                pixmap.dispose()
                imageRegion?.texture?.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Nearest)
            } else {
                imageProperty.width = 0f
                imageProperty.height = 0f
            }
            image = it
        }

        Bus.subscribe(SelectionCopyMessage::class.java) {
            val currentImage = image
            if (currentImage != null) {

                Context.selectBox.once {
                    val selectBox = it

                    if (selectBox.width != 0 && selectBox.height != 0) {
                        Clipboard.copy(currentImage,
                                selectBox.x,
                                selectBox.y,
                                selectBox.width,
                                selectBox.height)
                    } else {
                        Clipboard.copy(currentImage)
                    }

                }
            }
        }

        Context.selectBox.subscribe {
            updateSelectBoxContent(it)
        }
        Context.selectedImage.subscribe {
            updateSelectBoxContent(Context.selectBox.get())
        }
        Context.comparisonMode.subscribe {
            updateSelectBoxContent(Context.selectBox.get())
        }
    }

    fun updateSelectBoxContent(selectBox: Rect) {
        val selectedImage = Context.selectedImage.get()
        val mainImage = image
        if (mainImage != null && selectedImage != null &&
                selectBox.width > 0 && selectBox.height > 0 &&
                selectedImage.cols() == mainImage.cols() && selectedImage.rows() == mainImage.rows()) {

            val rect = selectBox.clone()
            rect.y = mainImage.rows() - rect.y - rect.height
            val mat = when (Context.comparisonMode.get()) {
                ComparisonMode.Image -> ImageProc.crop(selectedImage, rect)
                ComparisonMode.Diff -> ImageProc.diff(mainImage, selectedImage, rect)
            }

            val pixmap = ImageUtils.matToPixmap(mat)
            val texture = Texture(pixmap)

            pixmap.dispose()
            mat.release()
            if (overlayRegion.texture != null) {
                overlayRegion.texture.dispose()
            }

            overlayRegion.texture = texture
            overlayRegion.regionWidth = texture.width
            overlayRegion.regionHeight = texture.height

        } else {
            overlayRegion.texture = null
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
                    it.y * imageProperty.scale,
                    it.width * imageProperty.scale,
                    it.height * imageProperty.scale)
                    .hflip(imageProperty.height * imageProperty.scale + imageProperty.y)
        }

        if (Context.selectedImage.get() == null) {
            batch.color = selectOverlayRectColor
            batch.drawRect(selectDrawBox)
        }
        batch.color = selectOverlayBorderColor
        batch.drawRectBorder(selectDrawBox, 0.5f)

        // draw overlay
        batch.color = Color.WHITE
        if (overlayRegion.texture != null) {
            batch.draw(overlayRegion, selectDrawBox.x, selectDrawBox.y, selectDrawBox.width, selectDrawBox.height)
        }

        // draw zoom box
        Context.zoomBox.once {
            zoomDrawBox.set(
                    it.x * imageProperty.scale + imageProperty.x,
                    it.y * imageProperty.scale,
                    it.width * imageProperty.scale,
                    it.height * imageProperty.scale)
                    .hflip(imageProperty.height * imageProperty.scale + imageProperty.y)
        }
        batch.color = zoomOverlayRectColor
        batch.drawRect(zoomDrawBox)
        batch.color = zoomOverlayBorderColor
        batch.drawRectBorder(zoomDrawBox, 0.5f)

        // draw mouse cross
        if (Context.isShowCrosshair.get()) {
            batch.color = mouseCrossColor
            batch.drawLine(0f, mousePotition.y + 0.5f, width, mousePotition.y + 0.5f, 1f)
            batch.drawLine(mousePotition.x + 0.5f, 0f, mousePotition.x + 0.5f, height, 1f)
        }


        batch.color = prevColor

        super.draw(batch, parentAlpha)
    }

    fun selectAll() {
        Context.selectBox.update { it.set(0, 0, imageProperty.width.toInt(), imageProperty.height.toInt()) }
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
        var button = -1

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

            if (keycode == Input.Keys.PLUS) {
                zoom(imageViewer.width / 2,  imageViewer.height / 2, 1)
            }

            if (keycode == Input.Keys.MINUS) {
                zoom(imageViewer.width / 2,  imageViewer.height / 2, -1)
            }

            return super.keyDown(event, keycode)
        }

        override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
            initX = x
            initY = y
            prevX = x
            prevY = y
            this.button = button

            if (UIUtils.ctrl()) {
                mode = Mode.zoom
                Context.zoomBox.update {
                    it.set(
                        screenToPixelX(initX).toInt(),
                        screenToPixelY(initY).toInt(),
                        0, 0)
                            .hflip(imageProperty.height.toInt())
                }
            } else if (UIUtils.shift()) {
                mode = Mode.select
                Context.selectBox.update {
                    it.set(
                        screenToPixelX(initX).floor().toInt(),
                        screenToPixelY(initY).floor().toInt(),
                        0, 0)
                            .hflip(imageProperty.height.toInt())
                }
            } else {
                mode = Mode.move
            }

            if (event.stage != null) {
                event.stage.keyboardFocus = imageViewer
            }

            return true
        }

        override fun touchDragged(event: InputEvent, x: Float, y: Float, pointer: Int) {
            val dx = x - prevX
            val dy = y - prevY

            val x2 = imageProperty.width.toInt()
            val y2 = imageProperty.height.toInt()

            when(mode) {
                Mode.move -> {
                    if (button == Input.Buttons.LEFT) {
                        imageProperty.x += dx
                        imageProperty.y += dy
                    }
                }
                Mode.zoom -> {
                    Context.zoomBox.update {
                        it.set(
                                screenToPixelX(initX).toInt(),
                                screenToPixelY(initY).toInt(),
                                (screenToPixelX(x) - screenToPixelX(initX)).toInt(),
                                (screenToPixelY(y) - screenToPixelY(initY)).toInt())
                                .hflip(imageProperty.height.toInt())
                                .adjust()
                                .clamp(0, 0, x2, y2)
                    }
                }
                Mode.select -> {
                    Context.selectBox.update {
                        it.set(
                                screenToPixelX(initX).toInt(),
                                screenToPixelY(initY).toInt(),
                                (screenToPixelX(x) - screenToPixelX(initX)).toInt(),
                                (screenToPixelY(y) - screenToPixelY(initY)).toInt())
                                .hflip(imageProperty.height.toInt())
                                .adjust()
                                .clamp(0, 0, x2, y2)
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
                        if (it.width == 0 || it.height == 0) {
                            return@once
                        }
                        val widthScale = imageViewer.width / it.width
                        val heightScale = imageViewer.height / it.height
                        imageProperty.scale = Math.min(widthScale, heightScale)
                        imageProperty.x = (imageViewer.width - (it.x * 2 + it.width) * imageProperty.scale) / 2
                        imageProperty.y = -imageProperty.height * imageProperty.scale + it.height * imageProperty.scale - (imageViewer.height - (it.y * 2 + it.height) * imageProperty.scale) / 2

                        logScale = Math.log(imageProperty.scale.toDouble()).toFloat()

                        Context.zoomRate.update(imageProperty.scale)
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
            zoom(x, y, -amount)
            return true
        }

        fun cursorMoved(x: Float, y: Float) {
            val pixelX = screenToPixelX(x).toInt()
            val pixelY = screenToPixelY(y).toInt()

            Context.cursorPosition.update {
                val image = Context.mainImage.get()
                if (image != null) {
                    return@update it.set(pixelX, image.rows() - pixelY - 1)
                }
                return@update it.set(0, 0)
            }

            Context.cursorRGB.update {
                val image = Context.mainImage.get() ?: return@update intArrayOf()

                val channels = image.channels()
                val rgb = if (it.size == channels) it else IntArray(channels)

                val pixel = ImageProc.getPixel(image, pixelX, image.rows() - pixelY - 1)
                for (i in 0 until pixel.size) {
                    val value = pixel[i].toInt()
                    rgb[i] = if (value < 0) value + 256 else value
                }
                return@update rgb
            }

            mousePosition.x = x
            mousePosition.y = y
        }

        fun zoom(x: Float, y: Float, amount: Int) {

            val fracX = (x - imageProperty.x) / imageProperty.scale
            val fracY = (y - imageProperty.y) / imageProperty.scale

            logScale += amount / 16f
            imageProperty.scale = Math.exp(logScale.toDouble()).toFloat()

            imageProperty.x = x - fracX * imageProperty.scale
            imageProperty.y = y - fracY * imageProperty.scale

            Context.zoomRate.update(imageProperty.scale)
        }

    }
}