package kr.edoli.imview.image

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.glutils.ShaderProgram

class ViewerShaderBuilder {
    companion object {
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

        val fragShader = Gdx.files.internal("imageShader.frag").readString()
        val default_pixel_expression = "pow(p * contrast + brightness, 1.0 / gamma)"
    }

    var pixel_expression = default_pixel_expression

    fun build(): ShaderProgram {
        return ShaderProgram(vertexShader, fragShader.replace("%pixel_expression%", pixel_expression)).also {
            require(it.isCompiled) { "Error compiling shader: " + it.log }
        }
    }
}