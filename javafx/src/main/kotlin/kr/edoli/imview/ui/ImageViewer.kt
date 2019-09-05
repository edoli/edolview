package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.GL30
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.Sprite
import com.badlogic.gdx.graphics.g2d.SpriteBatch
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.badlogic.gdx.scenes.scene2d.Actor
import kr.edoli.imview.ImContext
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat

class ImageViewer : Actor() {
    var texture: Texture? = null
    var min = 0f
    var max = 0f

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
        ImContext.mainImage.subscribe { mat ->
            if (mat == null) return@subscribe

            val data = FloatArray((mat.total() * 3).toInt())
            mat.get(0, 0, data)

            min = Float.MAX_VALUE
            max = Float.MIN_VALUE
            data.forEach { value ->
                if (value > max) max = value
                if (value < min) min = value
            }

            val textureData = FloatTextureData(mat.width(), mat.height(), GL30.GL_RGB32F, GL30.GL_RGB, GL30.GL_FLOAT, false)
            textureData.prepare()
            textureData.buffer.put(data)
            textureData.buffer.position(0)

            texture = Texture(textureData)
        }
    }

    override fun draw(batch: Batch, parentAlpha: Float) {
        batch.end()

        batch.shader = shader
        batch.begin()
        shader.setUniformf("brightness", ImContext.imageBrightness.get())
        shader.setUniformf("contrast", ImContext.imageContrast.get())
        shader.setUniformf("gamma", ImContext.imageGamma.get())
        shader.setUniformf("min", min)
        shader.setUniformf("max", max)
        shader.setUniformi("normalize", ImContext.normalize.get().let { if (it) 1 else 0 })
        texture?.let {
            batch.draw(it, 0f, 0f, 500f, 500f)
        }
        batch.end()

        batch.shader = null
        batch.begin()
    }
}