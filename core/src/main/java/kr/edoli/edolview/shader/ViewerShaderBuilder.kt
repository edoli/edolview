package kr.edoli.edolview.shader

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import kr.edoli.edolview.util.Platform

const val vertexShader = """

attribute vec4 ${ShaderProgram.POSITION_ATTRIBUTE};
attribute vec4 ${ShaderProgram.COLOR_ATTRIBUTE};
attribute vec2 ${ShaderProgram.TEXCOORD_ATTRIBUTE}0;
uniform mat4 u_projTrans;
varying vec4 v_color;
varying vec2 v_texCoords;

void main()
{
   v_color = ${ShaderProgram.COLOR_ATTRIBUTE};
   v_color.a = v_color.a * (255.0/254.0);
   v_texCoords = ${ShaderProgram.TEXCOORD_ATTRIBUTE}0;
   gl_Position =  u_projTrans * ${ShaderProgram.POSITION_ATTRIBUTE};
}
"""
class ViewerShaderBuilder {
    companion object {
        fun getColormapNames(subDir: String): List<String> {
            val file = Gdx.files.internal("colormap/${subDir}")
            return file.list().map { it.name() }.filter {
                it.contains(".glsl")
            }?.map {
                it.replace(".glsl", "")
            }?.sortedBy { if (it == "color") "" else it } ?: listOf()
        }

        val fragShader: String = Gdx.files.internal("imageShader.frag").readString()
        const val default_pixel_expression = ""
    }

    private val rgbShaderStore = CacheBuilder.newBuilder()
            .removalListener<String, ExtendedShaderProgram> { it.value?.dispose() }
            .build(object : CacheLoader<String, ExtendedShaderProgram>() {
                override fun load(shaderName: String): ExtendedShaderProgram {
                    return build(shaderName, false)
                }
            })

    private val monoShaderStore = CacheBuilder.newBuilder()
            .removalListener<String, ExtendedShaderProgram> { it.value?.dispose() }
            .build(object : CacheLoader<String, ExtendedShaderProgram>() {
                override fun load(shaderName: String): ExtendedShaderProgram {
                    return build(shaderName, true)
                }
            })

    var customShader = ""

    // colormap before pixel processing for RGB images
    private val colorProcessRGBColormap = """
    vec3 t = %colormap_name%_colormap(tex.rgb);
    vec3 v;
    v.r = pow(t.r * pow(2.0, exposure) + offset, 1.0 / gamma);
    v.g = pow(t.g * pow(2.0, exposure) + offset, 1.0 / gamma);
    v.b = pow(t.b * pow(2.0, exposure) + offset, 1.0 / gamma);
    gl_FragColor = v_color * vec4(v, 1.0);
    gl_FragColor.a = alpha;
    """

    // colormap after pixel processing for Mono images
    private val colorProcessMonoColorMap = """
    float v = pow(tex.r * pow(2.0, exposure) + offset, 1.0 / gamma);
    v = clamp(v, 0.0, 1.0);
    vec3 color = %colormap_name%_colormap(v);
    gl_FragColor = vec4(color.r, color.g, color.b, alpha);
    """

    fun getRGB(colormapName: String) = rgbShaderStore.get(colormapName)
    fun getMono(colormapName: String) = monoShaderStore.get(colormapName)

    fun getCustom() = build("", false)
    fun clearCache() {
        rgbShaderStore.cleanUp()
        monoShaderStore.cleanUp()
    }

    private fun build(colormapName: String, isMono: Boolean): ExtendedShaderProgram {
        val colormapShaderCode = if (colormapName == "") "" else if (isMono)
            Gdx.files.internal("colormap/mono/${colormapName}.glsl").readString() else
            Gdx.files.internal("colormap/rgb/${colormapName}.glsl").readString()

        val shaderCode = if (customShader != "")
            fragShader
                .replace("%color_process%", customShader)
                .replace("%colormap_function%", "") else
            fragShader.replace("%color_process%",
                if (isMono)
                    colorProcessMonoColorMap.replace("%colormap_name%", colormapName) else
                    colorProcessRGBColormap.replace("%colormap_name%", colormapName))
                .replace("%colormap_function%", colormapShaderCode)

        return ExtendedShaderProgram(vertexShader, shaderCode, colormapShaderCode).also {
            if (!it.isCompiled) {
                Platform.showErrorMessage("Error compiling shader[${colormapName}]: " + it.log)
            }
        }
    }
}

