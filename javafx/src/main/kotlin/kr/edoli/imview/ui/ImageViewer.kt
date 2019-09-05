package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.GL30
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import kr.edoli.imview.ImContext
import kotlin.math.pow

class ImageViewer : Actor() {
    var texture: Texture? = null
    var textureRegion: TextureRegion? = null
    var min = 0f
    var max = 0f

    var imageX = 0f
    var imageY = 0f
    var imageScale = 1f

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

    init {
        // Mat -> Texture using [FloatTextureData]
        ImContext.mainImage.subscribe { mat ->
            if (mat == null) return@subscribe

            texture?.dispose()

            val numChannels = mat.channels()

            val data = FloatArray((mat.total() * numChannels).toInt())
            mat.get(0, 0, data)

            min = Float.MAX_VALUE
            max = Float.MIN_VALUE
            data.forEach { value ->
                if (value > max) max = value
                if (value < min) min = value
            }

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

            override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                touchDownX = x
                touchDownY = y

                touchDownImageX = imageX
                touchDownImageY = imageY

                stage.keyboardFocus = this@ImageViewer

                return true
            }

            override fun touchDragged(event: InputEvent, x: Float, y: Float, pointer: Int) {
                imageX = x - touchDownX + touchDownImageX
                imageY = y - touchDownY + touchDownImageY
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
    }

    private fun updateSmoothing() {
        val isSmoothing = ImContext.smoothing.get()
        if (isSmoothing) {
            texture?.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)
        } else {
            texture?.setFilter(Texture.TextureFilter.Nearest, Texture.TextureFilter.Nearest)
        }
    }

    override fun draw(batch: Batch, parentAlpha: Float) {
        batch.end()

        batch.color = Color.WHITE
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
        batch.end()

        batch.shader = null
        batch.begin()
    }
}