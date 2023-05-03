package kr.edoli.edolview.shader

import com.badlogic.gdx.graphics.glutils.ShaderProgram

class ExtendedShaderProgram(vertexShader: String, fragmentShader: String, colormapShader: String) :
    ShaderProgram(vertexShader, fragmentShader) {

        var absMax = false
            private set

        init {
            val escapeIndex = colormapShader.indexOf('\n')
            val firstLine = colormapShader.subSequence(2, escapeIndex).toString()

            val flags = firstLine.trim().replace(" ", "").split(',')

            if ("ABS_MAX" in flags) {
                absMax = true
            }
        }
}