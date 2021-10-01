package kr.edoli.imview.shader

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.glutils.ShaderProgram

class ViewerShaderBuilder {
    companion object {
        val fragShader: String = Gdx.files.internal("imageShader.frag").readString()
        const val default_pixel_expression = "pow(p * contrast + brightness, 1.0 / gamma)"
    }

    var extraCode = ""
    var pixelExpression = default_pixel_expression

    fun build(): ShaderProgram {
        return ShaderProgram(SpriteShaderBuilder.vertexShader, fragShader
                .replace("%extra_code%", extraCode.replace("%pixel_expression%", ""))
                .replace("%pixel_expression%", pixelExpression)).also {
            require(it.isCompiled) { "Error compiling shader: " + it.log }
        }
    }
}