package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.OrthographicCamera
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.SpriteBatch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import com.badlogic.gdx.graphics.glutils.FrameBuffer
import com.badlogic.gdx.graphics.glutils.ShapeRenderer
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.WidgetGroup
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import kr.edoli.imview.ImContext
import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.image.ImageConvert
import kr.edoli.imview.image.MarqueeUtils
import kr.edoli.imview.image.bound
import kr.edoli.imview.util.ceil
import kr.edoli.imview.util.floor
import kr.edoli.imview.util.reset
import kr.edoli.imview.util.toColorStr
import org.lwjgl.opengl.GL30
import org.opencv.core.Mat
import org.opencv.core.Point
import org.opencv.core.Rect
import java.nio.ByteBuffer
import kotlin.math.*

class ImageViewer : WidgetGroup() {
    var texture: Texture? = null
    var textureRegion: TextureRegion? = null

    var imageX = 0f
    var imageY = 0f
    var imageScale = 1f
    var imageWidth = 0
    var imageHeight = 0

    val handleSize = 3f

    val shapeRenderer = ShapeRenderer()

    val bufferCallbacks = ArrayList<(ByteArray) -> Unit>()

    val defaultShader = SpriteBatch.createDefaultShader()

    enum class DragMode {
        marquee, move
    }

    init {
        touchable = Touchable.enabled
        isTransform = true

        // Mat -> Texture using [FloatTextureData]
        ImContext.mainImage.subscribe(this, "Update texture") { mat ->
            Gdx.app.postRunnable {
                updateTexture(mat)
            }
        }

        ImContext.visibleChannel.subscribe(this, "Update texture") {
            Gdx.app.postRunnable {
                updateTexture(ImContext.mainImage.get())
            }
        }

        ImContext.smoothing.subscribe(this, "Update texture smoothing") { updateSmoothing() }

        // Drag listener
        addListener(object : InputListener() {
            var touchDownX = 0f
            var touchDownY = 0f

            var touchDownImageX = 0f
            var touchDownImageY = 0f

            var dragMode = DragMode.move

            val imageCoord = Vector2()
            val imageCoordB = Vector2()

            var marqueeOriginX = 0f
            var marqueeOriginY = 0f

            override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                if (button == Input.Buttons.RIGHT) {
                    return false
                }

                if (UIUtils.shift()) {
                    ImContext.marqueeBoxActive.update(true)
                    localToImageCoordinates(imageCoord.set(x, y))

                    ImContext.marqueeBox.update(Rect(imageCoord.x.toInt(), imageCoord.y.toInt(), 0, 0))
                    marqueeOriginX = x
                    marqueeOriginY = y
                    dragMode = DragMode.marquee
                } else {
                    dragMode = DragMode.move
                    val mousePoint = Point(x.toDouble(), y.toDouble())
                    val points = marqueeHandlePoints()
                    points.forEachIndexed { i, point ->
                        val r = handleSize + 2f
                        if (Rect((point.x - r).toInt(), (point.y - r).toInt(), (r * 2).toInt(), (r * 2).toInt()).contains(mousePoint)) {
                            dragMode = DragMode.marquee

                            val targetPoint = points[when (i) {
                                0 -> 3
                                1 -> 2
                                2 -> 1
                                3 -> 0
                                else -> 0
                            }]
                            marqueeOriginX = targetPoint.x.toFloat()
                            marqueeOriginY = targetPoint.y.toFloat()
                            return@forEachIndexed
                        }
                    }
                }

                touchDownX = x
                touchDownY = y

                touchDownImageX = imageX
                touchDownImageY = imageY

                stage.keyboardFocus = this@ImageViewer

                return true
            }

            override fun touchDragged(event: InputEvent, x: Float, y: Float, pointer: Int) {
                if (dragMode == DragMode.move) {
                    imageX = x - touchDownX + touchDownImageX
                    imageY = y - touchDownY + touchDownImageY
                } else if (dragMode == DragMode.marquee) {
                    localToImageCoordinates(imageCoord.set(marqueeOriginX, marqueeOriginY))
                    localToImageCoordinates(imageCoordB.set(x, y))

                    val x1 = max(min(imageCoord.x, imageCoordB.x).toInt(), 0)
                    val y1 = max(min(imageCoord.y, imageCoordB.y).toInt(), 0)

                    var width = max(imageCoord.x, imageCoordB.x).ceil().toInt() - x1
                    var height = max(imageCoord.y, imageCoordB.y).ceil().toInt() - y1

                    if (UIUtils.ctrl()) {
                        var size = max(width, height)
                        size = min(min(imageWidth - x1, imageHeight - y1), size)
                        width = size
                        height = size
                    }
                    val x2 = min(x1 + width, imageWidth)
                    val y2 = min(y1 + height, imageHeight)

                    ImContext.marqueeBox.update { rect ->
                        rect.apply {
                            this.x = x1
                            this.y = y1
                            this.width = x2 - x1
                            this.height = y2 - y1
                        }
                    }
                }
            }

            override fun mouseMoved(event: InputEvent, x: Float, y: Float): Boolean {
                stage.scrollFocus = this@ImageViewer
                localToImageCoordinates(imageCoord.set(x, y))
                ImContext.cursorPosition.update(Point2D(imageCoord.x.toDouble(), imageCoord.y.toDouble()))
                return false
            }

            override fun scrolled(event: InputEvent?, x: Float, y: Float, amountX: Float, amountY: Float): Boolean {
                ImContext.zoomLevel.update(ImContext.zoomLevel.get() - amountY.toInt())
                return super.scrolled(event, x, y, amountX, amountY)
            }

            override fun keyDown(event: InputEvent, keycode: Int): Boolean {
                if (keycode == Input.Keys.LEFT) {
                    ImContext.prevImage()
                    return true
                }
                if (keycode == Input.Keys.RIGHT) {
                    ImContext.nextImage()
                    return true
                }
                if (keycode == Input.Keys.C && UIUtils.ctrl()) {
                    ImContext.marqueeImage.get()?.let { mat ->
                        ClipboardUtils.putImage(mat)
                    }
                    return true
                }
                if (keycode == Input.Keys.ESCAPE) {
                    ImContext.marqueeBox.update { rect ->
                        rect.reset()
                    }
                    return true
                }
                if (keycode == Input.Keys.A && UIUtils.ctrl()) {
                    ImContext.mainImage.get()?.let { mat ->
                        ImContext.marqueeBox.update { rect ->
                            rect.x = 0
                            rect.y = 0
                            rect.width = mat.width()
                            rect.height = mat.height()
                            rect
                        }
                    }
                    return true
                }
                return false
            }
        })

        contextMenu {
            addMenu("Copy visible image") {
                bufferCallbacks.add { byteArray ->
                    val mat = ImContext.mainImage.get()
                    if (mat != null) {
                        ClipboardUtils.putImage(ImageConvert.byteArrayToBuffered(byteArray, mat.width(), mat.height(), 4))
                    }
                }
            }

            addMenu("Center cursor") {
                ImContext.centerCursor.onNext(true)
            }
            addMenu("Center image") {
                ImContext.centerImage.onNext(true)
            }
            addMenu("Fit image") {
                ImContext.fitImage.onNext(true)
            }

            if (ImContext.isValidMarquee) {
                addMenuDivider()
                addMenu("Center selection") {
                    ImContext.centerSelection.onNext(true)
                }
                addMenu("Fit selection") {
                    ImContext.fitSelection.onNext(true)
                }
            }

            addMenuDivider()
            addMenu("Copy cursor position") {
                ClipboardUtils.putString(ImContext.cursorPosition.get().toString())
            }
            addMenu("Copy cursor RGB") {
                ImContext.mainImageSpec.get()?.let { imageSpec ->
                    ClipboardUtils.putString(ImContext.cursorRGB.get().toColorStr(imageSpec.maxValue))
                }
            }

            if (ImContext.isValidMarquee) {
                addHorizontalDivider().pad(4f, 0f, 4f, 0f)
                addMenu("Copy selection bound") {
                    ClipboardUtils.putString(ImContext.marqueeBox.get().toString())
                }
                addMenu("Copy selection RGB") {
                    ImContext.mainImageSpec.get()?.let { imageSpec ->
                        ClipboardUtils.putString(ImContext.marqueeBoxRGB.get().toColorStr(imageSpec.maxValue))
                    }
                }
                addMenu("Save selected image") {
                    ImContext.marqueeImage.get()?.let { mat ->
                        MarqueeUtils.saveImage(false)
                    }
                }
                addMenu("Copy selected image") {
                    ImContext.marqueeImage.get()?.let { mat ->
                        ClipboardUtils.putImage(mat)
                    }
                }
                addMenu("Copy selected visible image") {
                    bufferCallbacks.add { byteArray ->
                        val mat = ImContext.mainImage.get()
                        if (mat != null) {
                            val box = ImContext.marqueeBox.get()

                            val matWidth = mat.width()
                            val rowSize = matWidth * 4

                            val croppedByteArray = ByteArray(box.height * box.width * 4)
                            val croppedRowSize = box.width * 4
                            for (i in 0 until box.height) {
                                val colStart = (matWidth * (box.y + i) + box.x) * 4
                                byteArray.copyInto(croppedByteArray, i * croppedRowSize, colStart, colStart + croppedRowSize)
                            }
                            ClipboardUtils.putImage(ImageConvert.byteArrayToBuffered(croppedByteArray, box.width, box.height, 4))
                        }
                    }
                }
            }
        }

        ImContext.zoomLevel.subscribe(this, "Update image zoom") { zoomLevel ->
            val mousePos = Vector2(Gdx.input.x.toFloat(), Gdx.input.y.toFloat())
            screenToLocalCoordinates(mousePos)

            val mousePosImageX = mousePos.x - imageX
            val mousePosImageY = mousePos.y - imageY
            val currentScale = imageScale

            val newScale = 1.1f.pow(zoomLevel)

            val newMousePosImageX = mousePosImageX * newScale / currentScale
            val newMousePosImageY = mousePosImageY * newScale / currentScale

            imageX = mousePos.x - newMousePosImageX
            imageY = mousePos.y - newMousePosImageY
            imageScale = newScale
        }

        ImContext.normalize.subscribe(this, "Calc normalization") {
            calcNormalization()
        }

        ImContext.centerCursor.subscribe {
            val mousePos = Vector2(Gdx.input.x.toFloat(), Gdx.input.y.toFloat())
            localToImageCoordinates(screenToLocalCoordinates(mousePos))
            centerRect(Rect(mousePos.x.toInt(), mousePos.y.toInt(), 0, 0))
        }
        ImContext.centerImage.subscribe {
            val mat = ImContext.mainImage.get() ?: return@subscribe
            centerRect(mat.bound())
        }
        ImContext.fitImage.subscribe {
            val mat = ImContext.mainImage.get() ?: return@subscribe
            fitRect(mat.bound())
        }

        ImContext.centerSelection.subscribe { centerMarquee() }
        ImContext.fitSelection.subscribe { fitMarquee() }
    }

    private fun updateTexture(mat: Mat?) {
        if (mat == null) return

        val visibleChannel = ImContext.visibleChannel.get()
        if (!TextureGenerator.isChanged(mat, visibleChannel)) return

        texture?.dispose()

        texture = TextureGenerator.load(mat, visibleChannel)
        textureRegion = TextureRegion(texture)

        // Statistics
        imageWidth = mat.width()
        imageHeight = mat.height()

        if (ImContext.normalize.get()) {
            calcNormalization()
        }

        updateSmoothing()
    }

    private fun calcNormalization() {
        val textureData = texture?.textureData as FloatTextureData?
        if (textureData != null) {
            val buffer = textureData.buffer
            buffer.position(0)

            var localMin = Float.MAX_VALUE
            var localMax = Float.MIN_VALUE

            while (buffer.hasRemaining()) {
                val value = buffer.get()
                if (value.isFinite()) {
                    if (value > localMax) localMax = value
                    if (value < localMin) localMin = value
                }
            }

            Gdx.app.postRunnable {
                ImContext.textureMin.update(localMin)
                ImContext.textureMax.update(localMax)
            }
        }
    }

    private fun centerMarquee() {
        if (!ImContext.isValidMarquee) {
            return
        }

        centerRect(ImContext.marqueeBox.get())
    }

    private fun fitMarquee() {
        if (!ImContext.isValidMarquee) {
            return
        }

        fitRect(ImContext.marqueeBox.get())
    }

    private fun fitRect(rect: Rect) {
        val vecA = Vector2(rect.x.toFloat(), (rect.y + rect.height).toFloat())
        val vecB = Vector2((rect.x + rect.width).toFloat(), rect.y.toFloat())

        imageToLocalCoordinates(vecA)
        imageToLocalCoordinates(vecB)
        imageScale *= height / (vecB.y - vecA.y)

        ImContext.zoomLevel.update(log(imageScale.toDouble(), 1.1).floor().toInt())

        centerRect(rect)
    }

    private fun centerRect(rect: Rect) {
        val vecA = Vector2(rect.x.toFloat(), (rect.y + rect.height).toFloat())
        val vecB = Vector2((rect.x + rect.width).toFloat(), rect.y.toFloat())

        imageToLocalCoordinates(vecA)
        imageToLocalCoordinates(vecB)

        val marqueeWidth = vecB.x - vecA.x
        val marqueeHeight = vecB.y - vecA.y
        imageX = imageX + (width - marqueeWidth) / 2 - vecA.x
        imageY = imageY + (height - marqueeHeight) / 2 - vecA.y
    }

    private fun localToImageCoordinates(vec: Vector2): Vector2 {
        vec.x = (vec.x - imageX) / imageScale
        vec.y = imageHeight - ((vec.y - imageY) / imageScale)
        return vec
    }

    private fun imageToLocalCoordinates(vec: Vector2): Vector2 {
        vec.x = (vec.x * imageScale) + imageX
        vec.y = ((imageHeight - vec.y) * imageScale) + imageY
        return vec
    }

    private fun updateSmoothing() {
        val isSmoothing = ImContext.smoothing.get()
        if (isSmoothing) {
            texture?.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)
        } else {
            texture?.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Nearest)
        }
    }

    private fun marqueeHandlePoints(): Array<Point2D> {
        val marqueeBox = ImContext.marqueeBox.get()
        val vecA = imageToLocalCoordinates(Vector2(marqueeBox.x.toFloat(), marqueeBox.y.toFloat()))
        val vecB = imageToLocalCoordinates(Vector2((marqueeBox.x + marqueeBox.width).toFloat(), (marqueeBox.y + marqueeBox.height).toFloat()))

        return arrayOf(
                Point2D(vecA.x.toDouble(), vecA.y.toDouble()),
                Point2D(vecA.x.toDouble(), vecB.y.toDouble()),
                Point2D(vecB.x.toDouble(), vecA.y.toDouble()),
                Point2D(vecB.x.toDouble(), vecB.y.toDouble())
//
//                Point2D(((vecA.x + vecB.x) / 2).toDouble(), vecA.y.toDouble()),
//                Point2D(((vecA.x + vecB.x) / 2).toDouble(), vecB.y.toDouble()),
//                Point2D(vecA.x.toDouble(), ((vecA.y + vecB.y) / 2).toDouble()),
//                Point2D(vecB.x.toDouble(), ((vecA.y + vecB.y) / 2).toDouble())
        )
    }

    override fun drawChildren(batch: Batch, parentAlpha: Float) {
        batch.color = Color.WHITE
        batch.end()

        if (bufferCallbacks.isNotEmpty()) {
            val buffer = drawBuffer()
            if (buffer != null) {
                bufferCallbacks.forEach { it(buffer) }
            }
            bufferCallbacks.clear()
        }

        drawImage(batch, imageX, imageY, imageScale)

        // draw marquee box
        val marqueeBox = ImContext.marqueeBox.get()
        val vecA = imageToLocalCoordinates(Vector2(marqueeBox.x.toFloat(), marqueeBox.y.toFloat()))
        val vecB = imageToLocalCoordinates(Vector2((marqueeBox.x + marqueeBox.width).toFloat(), (marqueeBox.y + marqueeBox.height).toFloat()))

        shapeRenderer.projectionMatrix = batch.projectionMatrix
        shapeRenderer.transformMatrix = batch.transformMatrix
        shapeRenderer.color = Color.WHITE
        shapeRenderer.begin(ShapeRenderer.ShapeType.Line)
        shapeRenderer.rect(vecA.x, vecA.y, vecB.x - vecA.x, vecB.y - vecA.y)
        shapeRenderer.end()

        val maxSize = max(abs(vecA.x - vecB.x), abs(vecA.y - vecB.y))
        if (maxSize > 16f) {
            shapeRenderer.begin(ShapeRenderer.ShapeType.Filled)

            for (point in marqueeHandlePoints()) {
                val x = point.x
                val y = point.y
                shapeRenderer.rect((x - handleSize).toFloat(), (y - handleSize).toFloat(), handleSize * 2, handleSize * 2)
            }
            shapeRenderer.end()
        }

        // draw crosshair
        if (ImContext.isShowCrosshair.get()) {
            shapeRenderer.color = Color.GREEN

            val mousePos = Vector2(Gdx.input.x.toFloat(), Gdx.input.y.toFloat())
            screenToLocalCoordinates(mousePos)
            shapeRenderer.begin(ShapeRenderer.ShapeType.Line)
            shapeRenderer.line(x, mousePos.y, width, mousePos.y)
            shapeRenderer.line(mousePos.x, y, mousePos.x, height)
            shapeRenderer.end()
        }

        batch.begin()
    }

    fun drawImage(batch: Batch, localX: Float, localY: Float, localScale: Float) {
        batch.begin()
        if (ImContext.enableDisplayProfile.get()) {
            val shader = ImContext.viewerShader.get()
            batch.shader = shader
            shader.setUniformf("brightness", ImContext.imageBrightness.get())
            shader.setUniformf("contrast", ImContext.imageContrast.get())
            shader.setUniformf("gamma", ImContext.imageGamma.get())
            if (ImContext.normalize.get()) {
                shader.setUniformf("min", ImContext.textureMin.get())
                shader.setUniformf("max", ImContext.textureMax.get())
            } else {
                shader.setUniformf("min", ImContext.displayMin.get())
                shader.setUniformf("max", ImContext.displayMax.get())
            }
            if (ImContext.visibleChannel.get() != 0 || ImContext.mainImage.get()?.channels() == 1) {
                shader.setUniformi("colormap", ImContext.imageColormap.currentIndex)
            } else {
                shader.setUniformi("colormap", 0)
            }
//            shader.setUniformi("normalize", ImContext.normalize.get().let { if (it) 1 else 0 })
            textureRegion?.let { region ->
                batch.draw(region, localX, localY, 0f, 0f,
                        region.regionWidth.toFloat(), region.regionHeight.toFloat(), localScale, localScale, 0f)
            }

            batch.shader = defaultShader
        } else {
            textureRegion?.let { region ->
                batch.draw(region, localX, localY, 0f, 0f,
                        region.regionWidth.toFloat(), region.regionHeight.toFloat(), localScale, localScale, 0f)
            }
        }
        batch.end()
    }

    fun drawBuffer(): ByteArray? {
        val batch = SpriteBatch()
        val image = ImContext.mainImage.get()
        if (image != null) {
            val frameBuffer = FrameBuffer(texture!!.textureData.format, image.width(), image.height(), false)
            val camera = OrthographicCamera(image.width().toFloat(), image.height().toFloat())
            camera.position.set(image.width().toFloat() / 2f, image.height().toFloat() / 2f, 0f)
            camera.update()
            frameBuffer.begin()
            batch.projectionMatrix = camera.combined
            Gdx.gl.glViewport(0, 0, image.width(), image.height())
            Gdx.gl.glDisable(GL30.GL_SCISSOR_TEST)

            drawImage(batch, 0f, 0f, 1f)

            val texture = frameBuffer.colorBufferTexture
            texture.bind()
            val channels = 4
            val data = ByteBuffer.allocateDirect(texture.width * texture.height * channels)
            GL30.glGetTexImage(GL30.GL_TEXTURE_2D, 0, GL30.GL_RGBA, GL30.GL_UNSIGNED_BYTE, data)
            val error = GL30.glGetError()

            if (error != GL30.GL_NO_ERROR) {
                Gdx.app.error("Texture dump", "Get error: $error")
            }

            data.position(0)
            val byteArray = ByteArray(data.remaining())
            val width = frameBuffer.width
            val height = frameBuffer.height

            for (i in 0 until height) {
                data.get(byteArray, width * (height - i - 1) * channels, width * channels)
            }

            frameBuffer.end()
            Gdx.gl.glEnable(GL30.GL_SCISSOR_TEST)

            return byteArray
        }
        return null
    }
}