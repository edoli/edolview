package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.OrthographicCamera
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.SpriteBatch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import com.badlogic.gdx.graphics.glutils.FrameBuffer
import com.badlogic.gdx.graphics.glutils.ShaderProgram
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
import kr.edoli.imview.image.bound
import kr.edoli.imview.util.ceil
import kr.edoli.imview.util.toColorStr
import org.lwjgl.opengl.GL30
import org.opencv.core.Mat
import org.opencv.core.Rect
import java.nio.ByteBuffer
import kotlin.math.log
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow

class ImageViewer : WidgetGroup() {
    var texture: Texture? = null
    var textureRegion: TextureRegion? = null
    var min = 0.0f
    var max = 0.0f

    var imageX = 0f
    var imageY = 0f
    var imageScale = 1f
    var imageWidth = 0
    var imageHeight = 0

    val shapeRenderer = ShapeRenderer()

    val bufferCallbacks = ArrayList<(ByteArray) -> Unit>()

    val vertexShader = ("attribute vec4 " + ShaderProgram.POSITION_ATTRIBUTE + ";\n" //
            + "attribute vec4 " + ShaderProgram.COLOR_ATTRIBUTE + ";\n" //
            + "attribute vec2 " + ShaderProgram.TEXCOORD_ATTRIBUTE + "0;\n" //
            + "uniform mat4 u_projTrans;\n" //
            + "varying vec4 v_color;\n" //
            + "varying vec2 v_texCoords;\n" //
            + "\n" //
            + "void main()\n" //
            + "{\n" //
            + "   v_color = " + ShaderProgram.COLOR_ATTRIBUTE + ";\n" //
            + "   v_color.a = v_color.a * (255.0/254.0);\n" //
            + "   v_texCoords = " + ShaderProgram.TEXCOORD_ATTRIBUTE + "0;\n" //
            + "   gl_Position =  u_projTrans * " + ShaderProgram.POSITION_ATTRIBUTE + ";\n" //
            + "}\n")

    val shader = ShaderProgram(vertexShader, Gdx.files.internal("imageShader.frag").readString()).also {
        require(it.isCompiled) { "Error compiling shader: " + it.log }
    }
    val defaultShader = SpriteBatch.createDefaultShader()

    enum class DragMode {
        marquee, move
    }

    init {
        touchable = Touchable.enabled
        isTransform = true
        // scroll focus
        Gdx.app.postRunnable {
            stage.scrollFocus = this
        }

        // Mat -> Texture using [FloatTextureData]
        ImContext.mainImage.subscribe { mat ->
            updateTexture(mat)
        }
        ImContext.visibleChannel.subscribe {
            updateTexture(ImContext.mainImage.get())
        }

        ImContext.smoothing.subscribe { updateSmoothing() }

        // Drag listener
        addListener(object : InputListener() {
            var touchDownX = 0f
            var touchDownY = 0f

            var touchDownImageX = 0f
            var touchDownImageY = 0f

            var dragMode = DragMode.move

            val imageCoord = Vector2()
            val imageCoordB = Vector2()


            override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                if (button == Input.Buttons.RIGHT) {
                    return false
                }

                dragMode = if (UIUtils.shift()) {
                    ImContext.marqueeBoxActive.update(true)
                    localToImageCoordinates(imageCoord.set(x, y))

                    ImContext.marqueeBox.update(Rect(imageCoord.x.toInt(), imageCoord.y.toInt(), 0, 0))
                    DragMode.marquee
                } else {
                    DragMode.move
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
                    localToImageCoordinates(imageCoord.set(touchDownX, touchDownY))
                    localToImageCoordinates(imageCoordB.set(x, y))

                    val x1 = max(min(imageCoord.x, imageCoordB.x).toInt(), 0)
                    val y1 = max(min(imageCoord.y, imageCoordB.y).toInt(), 0)
                    val x2 = min(max(imageCoord.x, imageCoordB.x).ceil().toInt(), imageWidth)
                    val y2 = min(max(imageCoord.y, imageCoordB.y).ceil().toInt(), imageHeight)

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

            override fun scrolled(event: InputEvent, x: Float, y: Float, amount: Int): Boolean {
                ImContext.zoomLevel.update(ImContext.zoomLevel.get() - amount)
                return super.scrolled(event, x, y, amount)
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
                addMenu("Copy selected image") {
                    ImContext.marqueeImage.get()?.let { mat ->
                        ClipboardUtils.putImage(mat)
                    }
                }
            }
        }

        ImContext.zoomLevel.subscribe { zoomLevel ->
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

        ImContext.normalize.subscribe {
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
        min = Float.MAX_VALUE
        max = Float.MIN_VALUE

        calcNormalization()

        updateSmoothing()
    }

    private fun calcNormalization() {
        if (ImContext.normalize.get() && max == Float.MIN_VALUE && min == Float.MAX_VALUE) {
            val textureData = texture?.textureData as FloatTextureData
            val buffer = textureData.buffer
            buffer.position(0)

            var localMin = Float.MAX_VALUE
            var localMax = Float.MIN_VALUE

            while (buffer.hasRemaining()) {
                val value = buffer.get()
                if (value > localMax) localMax = value
                if (value < localMin) localMin = value
            }

            Gdx.app.postRunnable {
                min = localMin
                max = localMax
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

        ImContext.zoomLevel.update(log(imageScale.toDouble(), 1.1).toInt())

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

        shapeRenderer.projectionMatrix = batch.projectionMatrix
        shapeRenderer.transformMatrix = batch.transformMatrix
        shapeRenderer.begin(ShapeRenderer.ShapeType.Line)
        val marqueeBox = ImContext.marqueeBox.get()
        val vecA = imageToLocalCoordinates(Vector2(marqueeBox.x.toFloat(), marqueeBox.y.toFloat()))
        val vecB = imageToLocalCoordinates(Vector2((marqueeBox.x + marqueeBox.width).toFloat(), (marqueeBox.y + marqueeBox.height).toFloat()))
        shapeRenderer.rect(vecA.x, vecA.y, vecB.x - vecA.x, vecB.y - vecA.y)
        shapeRenderer.end()

        batch.begin()
    }

    fun drawImage(batch: Batch, localX: Float, localY: Float, localScale: Float) {
        batch.begin()
        if (ImContext.enableDisplayProfile.get()) {
            batch.shader = shader
            shader.setUniformf("brightness", ImContext.imageBrightness.get())
            shader.setUniformf("contrast", ImContext.imageContrast.get())
            shader.setUniformf("gamma", ImContext.imageGamma.get())
            shader.setUniformf("min", min)
            shader.setUniformf("max", max)
            if (ImContext.visibleChannel.get() != 0 || ImContext.mainImage.get()?.channels() == 1) {
                shader.setUniformi("colormap", ImContext.imageColormap.currentIndex)
            } else {
                shader.setUniformi("colormap", 0)
            }
            shader.setUniformi("normalize", ImContext.normalize.get().let { if (it) 1 else 0 })
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
            batch.projectionMatrix = camera.combined
            frameBuffer.begin()

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

            return byteArray
        }
        return null
    }
}