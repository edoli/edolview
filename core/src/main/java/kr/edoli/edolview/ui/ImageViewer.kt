package kr.edoli.edolview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.*
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.SpriteBatch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.graphics.glutils.FrameBuffer
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.badlogic.gdx.graphics.glutils.ShapeRenderer
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.WidgetGroup
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import com.badlogic.gdx.utils.BufferUtils
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.geom.Point2D
import kr.edoli.edolview.image.*
import kr.edoli.edolview.shader.BackgroundShaderBuilder
import kr.edoli.edolview.ui.custom.MyInputListener
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.util.*
import org.opencv.core.Mat
import org.opencv.core.Point
import org.opencv.core.Rect
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

    val backgroundShader = BackgroundShaderBuilder().build(Gdx.files.internal("backgroundShader.frag").readString())
    val backgroundMesh = Mesh(false, 4, 6,
        VertexAttribute(VertexAttributes.Usage.Position, 2, ShaderProgram.POSITION_ATTRIBUTE)).apply {
        setIndices(shortArrayOf(0, 1, 2, 0, 2, 3))
    }

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
            updateTexture(mat)
        }

        ImContext.visibleChannel.subscribeValue(this, "Update texture") {
            updateTexture(ImContext.mainImage.get())
        }

        ImContext.smoothing.subscribe(this, "Update texture smoothing") { updateSmoothing() }

        // Drag listener
        addListener(object : MyInputListener() {
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

                    val x1n = if (imageCoordB.x > imageCoord.x) imageCoord.x.floor() else imageCoord.x.ceil()
                    val y1n = if (imageCoordB.y > imageCoord.y) imageCoord.y.floor() else imageCoord.y.ceil()

                    val x1 = min(max(x1n.toInt(), 0), imageWidth)
                    val y1 = min(max(y1n.toInt(), 0), imageHeight)

                    val x2n = if (imageCoordB.x > imageCoord.x) imageCoordB.x.ceil() else imageCoordB.x.floor()
                    val y2n = if (imageCoordB.y > imageCoord.y) imageCoordB.y.ceil() else imageCoordB.y.floor()

                    val x2 = min(max(x2n.toInt(), 0), imageWidth)
                    val y2 = min(max(y2n.toInt(), 0), imageHeight)

                    val xf = if (imageCoordB.x > imageCoord.x) 1 else -1
                    val yf = if (imageCoordB.y > imageCoord.y) 1 else -1

                    var width = (x2 - x1).absoluteValue
                    var height = (y2 - y1).absoluteValue

                    if (UIUtils.ctrl()) {
                        var size = max(width, height)
                        val xMax = if (xf > 0) imageWidth - x1 else x1
                        val yMax = if (yf > 0) imageHeight - y1 else y1

                        size = min(min(xMax, yMax), size)
                        width = size
                        height = size
                    }

                    val boxX = min(x1, x1 + width * xf)
                    val boxY = min(y1, y1 + height * yf)

                    ImContext.marqueeBox.update { rect ->
                        rect.apply {
                            this.x = boxX
                            this.y = boxY
                            this.width = width
                            this.height = height
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
                ImContext.zoom.update(1.1f.pow(log(ImContext.zoom.get(), 1.1f) - amountY.toInt()))
                return super.scrolled(event, x, y, amountX, amountY)
            }

            override fun keyLongDown(keycode: Int) {
                super.keyLongDown(keycode)
                if (keycode == Input.Keys.LEFT) {
                    ImContext.mainAssetNavigator.update(-1)
                }
                if (keycode == Input.Keys.RIGHT) {
                    ImContext.mainAssetNavigator.update(1)
                }
            }

            override fun keyDown(event: InputEvent, keycode: Int): Boolean {
                super.keyDown(event, keycode)
                if (keycode == Input.Keys.LEFT) {
                    ImContext.mainAssetNavigator.update(-1)
                    return true
                }
                if (keycode == Input.Keys.RIGHT) {
                    ImContext.mainAssetNavigator.update(1)
                    return true
                }
                if (keycode == Input.Keys.C && UIUtils.ctrl()) {
                    bufferCallbacks.add { byteArray ->
                        val mat = ImContext.mainImage.get()
                        if (mat != null) {
                            val box = ImContext.marqueeBox.get()

                            if (ImContext.isValidMarquee) {
                                val matWidth = mat.width()

                                val croppedByteArray = ByteArray(box.height * box.width * 4)
                                val croppedRowSize = box.width * 4
                                for (i in 0 until box.height) {
                                    val colStart = (matWidth * (box.y + i) + box.x) * 4
                                    byteArray.copyInto(croppedByteArray, i * croppedRowSize, colStart, colStart + croppedRowSize)
                                }
                                ClipboardUtils.putImage(ImageConvert.byteArrayToBuffered(croppedByteArray, box.width, box.height, 4))
                            } else {
                                ClipboardUtils.putImage(ImageConvert.byteArrayToBuffered(byteArray, mat.width(), mat.height(), 4))
                            }
                        }
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
            val cursorPosition = ImContext.cursorPosition.get().toString()
            val cursorRGB = ImContext.mainImageSpec.get()?.let { imageSpec ->
                ImContext.cursorRGB.get().toColorStr(imageSpec)
            }

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
                ClipboardUtils.putString(cursorPosition)
            }
            if (cursorRGB != null) {
                addMenu("Copy cursor RGB") {
                    ClipboardUtils.putString(cursorRGB)
                }
            }

            if (ImContext.isValidMarquee) {
                addHorizontalDivider().pad(4f, 0f, 4f, 0f)
                addMenu("Copy selection bound") {
                    ClipboardUtils.putString(ImContext.marqueeBox.get().toString())
                }
                addMenu("Copy selection RGB") {
                    ImContext.mainImageSpec.get()?.let { imageSpec ->
                        ClipboardUtils.putString(ImContext.marqueeBoxRGB.get().toColorStr(imageSpec))
                    }
                }
                addMenu("Save selected image") {
                    ImContext.marqueeImage.get()?.let { _ ->
                        MarqueeUtils.saveImage()
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

        ImContext.zoom.subscribe(this, "Update image zoom") { zoom ->
            val zoomCenter = ImContext.zoomCenter.get() ?: Vector2(Gdx.input.x.toFloat(), Gdx.input.y.toFloat())
            screenToLocalCoordinates(zoomCenter)

            val mousePosImageX = zoomCenter.x - imageX
            val mousePosImageY = zoomCenter.y - imageY
            val currentScale = imageScale

            val newScale = zoom

            val newMousePosImageX = mousePosImageX * newScale / currentScale
            val newMousePosImageY = mousePosImageY * newScale / currentScale

            imageX = zoomCenter.x - newMousePosImageX
            imageY = zoomCenter.y - newMousePosImageY
            imageScale = newScale
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
        if (!TextureGenerator.isChanged(mat, visibleChannel ?: 0)) return

        texture?.dispose()

        texture = TextureGenerator.load(mat, visibleChannel ?: 0)
        textureRegion = TextureRegion(texture)

        // Statistics
        imageWidth = mat.width()
        imageHeight = mat.height()

        updateSmoothing()
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

        val viewerAspectRatio = width / height
        val rectAspectRatio = rect.width.toFloat() / rect.height

        val newImageScale = if (rectAspectRatio > viewerAspectRatio) {
            imageScale * width / (vecB.x - vecA.x)
        } else {
            imageScale * height / (vecB.y - vecA.y)
        }

        ImContext.zoom.update(newImageScale)

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
            texture?.setFilter(Texture.TextureFilter.Nearest, Texture.TextureFilter.Nearest)
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
        )
    }

    override fun drawChildren(batch: Batch, parentAlpha: Float) {
        batch.color = Color.WHITE
        batch.end()

        // draw background
        if (ImContext.isShowBackground.get()) {
            backgroundShader.bind()
            val combined = batch.projectionMatrix.cpy().mul(batch.transformMatrix)
            backgroundShader.setUniformMatrix("u_projTrans", combined)
            backgroundShader.setUniformf("u_grid_size", 8.0f)
            backgroundShader.setUniform2fv("u_translate", floatArrayOf(imageX, imageY), 0, 2)
            backgroundShader.setUniform4fv("grid_color_a", Colors.background.toFloatArray(), 0, 4)
            backgroundShader.setUniform4fv("grid_color_b", Colors.backgroundDown.toFloatArray(), 0, 4)
            backgroundMesh.setVertices(floatArrayOf(x, y, x + width, y, x + width, y + height, x, y + height))
            backgroundMesh.render(backgroundShader, GL20.GL_TRIANGLES)
        }

        // draw image
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
        Gdx.gl.glEnable(GL30.GL_BLEND)
        Gdx.gl.glBlendFunc(GL30.GL_ONE_MINUS_DST_COLOR, GL30.GL_ZERO)
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
        Gdx.gl.glBlendFunc(GL30.GL_SRC_ALPHA, GL30.GL_ONE_MINUS_SRC_ALPHA)
        Gdx.gl.glDisable(GL30.GL_BLEND)

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
        textureRegion?.let { region ->
            if (ImContext.enableDisplayProfile.get()) {
                val shader = ImContext.viewerShader.get()
                batch.shader = shader
                shader.setUniformi("width", region.regionWidth)
                shader.setUniformi("height", region.regionHeight)

                shader.setUniformf("r_scale", if (ImContext.invertR.get()) -1.0f else 1.0f)
                shader.setUniformf("g_scale", if (ImContext.invertG.get()) -1.0f else 1.0f)
                shader.setUniformf("b_scale", if (ImContext.invertB.get()) -1.0f else 1.0f)

                shader.setUniformi("is_inverse", if (ImContext.inverse.get()) 1 else 0)
                shader.setUniformf("offset", ImContext.imageOffset.get())
                shader.setUniformf("exposure", ImContext.imageExposure.get())
                shader.setUniformf("gamma", ImContext.imageGamma.get())
                if (ImContext.normalize.get()) {

                    var minMax = ImContext.mainImageSpec.get()!!.minMax

                    if (shader.absMax) {
                        val absMax = max(minMax.first.absoluteValue, minMax.second.absoluteValue)
                        minMax = Pair(0.0, absMax)
                    }

                    if (ImContext.inverse.get()) {
                        shader.setUniformf("minV", 1.0f / minMax.second.toFloat())
                        shader.setUniformf("maxV", 1.0f / minMax.first.toFloat())
                    } else {
                        shader.setUniformf("minV", minMax.first.toFloat())
                        shader.setUniformf("maxV", minMax.second.toFloat())
                    }
                } else {
                    shader.setUniformf("minV", ImContext.displayMin.get())
                    shader.setUniformf("maxV", ImContext.displayMax.get())
                }
                batch.draw(region, localX, localY, 0f, 0f,
                    region.regionWidth.toFloat(), region.regionHeight.toFloat(), localScale, localScale, 0f)

                batch.shader = defaultShader
            } else {
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

            Gdx.gl.glPixelStorei(GL20.GL_PACK_ALIGNMENT, 1)
            val pixels = BufferUtils.newByteBuffer(texture.width * texture.height * channels)
            Gdx.gl.glReadPixels(0, 0, texture.width, texture.height, GL20.GL_RGBA, GL20.GL_UNSIGNED_BYTE, pixels)

            val error = Gdx.gl.glGetError()
            if (error != GL30.GL_NO_ERROR) {
                Gdx.app.error("Texture dump", "Get error: $error")
            }

            pixels.position(0)
            val byteArray = ByteArray(pixels.remaining())
            val width = frameBuffer.width
            val height = frameBuffer.height

            for (i in 0 until height) {
                pixels.get(byteArray, width * (height - i - 1) * channels, width * channels)
            }

            frameBuffer.end()
            Gdx.gl.glEnable(GL30.GL_SCISSOR_TEST)

            return byteArray
        }
        return null
    }
}