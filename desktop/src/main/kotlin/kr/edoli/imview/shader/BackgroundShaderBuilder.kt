package kr.edoli.imview.shader

import com.badlogic.gdx.graphics.glutils.ShaderProgram

class BackgroundShaderBuilder {
    companion object {
        const val vertexShader = ("attribute vec4 " + ShaderProgram.POSITION_ATTRIBUTE + ";\n" //
                + "uniform mat4 u_projTrans;\n" //
                + "uniform vec2 u_translate;\n" //
                + "varying vec2 v_position;\n" //
                + "\n" //
                + "void main()\n" //
                + "{\n" //
                + "   v_position = " + ShaderProgram.POSITION_ATTRIBUTE + ".xy - u_translate;\n" //
                + "   gl_Position =  u_projTrans * " + ShaderProgram.POSITION_ATTRIBUTE + ";\n" //
                + "}\n")
    }

    fun build(fragShader: String): ShaderProgram {
        return ShaderProgram(vertexShader, fragShader).also {
            require(it.isCompiled) { "Error compiling shader: " + it.log }
        }
    }
}