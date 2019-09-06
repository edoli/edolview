package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.GL30
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.SpriteBatch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.badlogic.gdx.graphics.glutils.ShapeRenderer
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Group
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import kr.edoli.imview.ImContext
import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.util.ceil
import kr.edoli.imview.util.reset
import org.opencv.core.CvType
import org.opencv.core.Rect
import kotlin.math.log
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow

class ImageViewer : Group() {
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
            if (mat == null) return@subscribe

            texture?.dispose()

            val numChannels = mat.channels()

            val data = FloatArray((mat.total() * numChannels).toInt())
            val tmpMat = mat.clone()

            when (numChannels) {
                1 -> tmpMat.convertTo(tmpMat, CvType.CV_32FC1)
                2 -> tmpMat.convertTo(tmpMat, CvType.CV_32FC2)
                3 -> tmpMat.convertTo(tmpMat, CvType.CV_32FC3)
                4 -> tmpMat.convertTo(tmpMat, CvType.CV_32FC4)
            }

            tmpMat.get(0, 0, data)

            imageWidth = mat.width()
            imageHeight = mat.height()
            min = Float.MAX_VALUE
            max = Float.MIN_VALUE

            calcNormalization()

            val internalFormat = when (numChannels) {
                1 -> GL30.GL_RGB32F
                2 -> GL30.GL_RGB32F
                3 -> GL30.GL_RGB32F
                4 -> GL30.GL_RGBA32F
                else -> GL30.GL_RGB32F
            }

            val format = when (numChannels) {
                1 -> GL30.GL_RGB
                2 -> GL30.GL_RGB
                3 -> GL30.GL_RGB
                4 -> GL30.GL_RGBA
                else -> GL30.GL_RGB
            }

            val textureData = FloatTextureData(mat.width(), mat.height(), internalFormat, format, GL30.GL_FLOAT, false)
            textureData.prepare()
            if (numChannels == 1) {
                val buffer = FloatArray(data.size * 3)
                var i = 0
                data.forEach { v -> repeat(3) { buffer[i++] = v } }
                textureData.buffer.put(buffer)
            } else {
                textureData.buffer.put(data)
            }
            textureData.buffer.position(0)

            texture = Texture(textureData)
            updateSmoothing()
            textureRegion = TextureRegion(texture)
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
                localToImageCoordinates(imageCoord.set(x, y))
                ImContext.cursorPosition.update(Point2D(imageCoord.x.toDouble(), imageCoord.y.toDouble()))
                return false
            }

            override fun scrolled(event: InputEvent, x: Float, y: Float, amount: Int): Boolean {
                ImContext.zoomLevel.update(ImContext.zoomLevel.get() - amount)
                return super.scrolled(event, x, y, amount)
            }

            override fun keyDown(event: InputEvent, keycode: Int): Boolean {
                if (keycode == Input.Keys.LEFT) {
                    ImContext.prevImage()
                }
                if (keycode == Input.Keys.RIGHT) {
                    ImContext.nextImage()
                }
                if (keycode == Input.Keys.C && UIUtils.ctrl()) {
                    ImContext.marqueeImage.get()?.let { mat ->
                        ClipboardUtils.putImage(mat)
                    }
                }
                if (keycode == Input.Keys.ESCAPE) {
                    ImContext.marqueeBox.update { rect ->
                        rect.reset()
                    }
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
                }
                return super.keyDown(event, keycode)
            }
        })

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

        ImContext.centerImage.subscribe {
            centerMarquee()
        }

        ImContext.normalize.subscribe {
            calcNormalization()
        }

        ImContext.fitImage.subscribe {
            if (!ImContext.isValidMarquee) {
                return@subscribe
            }

            val marqueeBox = ImContext.marqueeBox.get()

            val vecA = Vector2(marqueeBox.x.toFloat(), (marqueeBox.y + marqueeBox.height).toFloat())
            val vecB = Vector2((marqueeBox.x + marqueeBox.width).toFloat(), marqueeBox.y.toFloat())

            imageToLocalCoordinates(vecA)
            imageToLocalCoordinates(vecB)
            imageScale *= height / (vecB.y - vecA.y)

            ImContext.zoomLevel.update(log(imageScale.toDouble(), 1.1).toInt())

            centerMarquee()
        }
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

        val marqueeBox = ImContext.marqueeBox.get()
        val vecA = Vector2(marqueeBox.x.toFloat(), (marqueeBox.y + marqueeBox.height).toFloat())
        val vecB = Vector2((marqueeBox.x + marqueeBox.width).toFloat(), marqueeBox.y.toFloat())

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
        if (ImContext.enableProfile.get()) {
            batch.end()

            batch.shader = shader
            batch.begin()
            shader.setUniformf("brightness", ImContext.imageBrightness.get())
            shader.setUniformf("contrast", ImContext.imageContrast.get())
            shader.setUniformf("gamma", ImContext.imageGamma.get())
            shader.setUniformf("min", min)
            shader.setUniformf("max", max)
            shader.setUniformi("normalize", ImContext.normalize.get().let { if (it) 1 else 0 })
            textureRegion?.let { region ->
                batch.draw(region, imageX, imageY, 0f, 0f,
                        region.regionWidth.toFloat(), region.regionHeight.toFloat(), imageScale, imageScale, 0f)
            }

            batch.shader = defaultShader
        } else {
            textureRegion?.let { region ->
                batch.draw(region, imageX, imageY, 0f, 0f,
                        region.regionWidth.toFloat(), region.regionHeight.toFloat(), imageScale, imageScale, 0f)
            }
        }
        batch.end()


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
}