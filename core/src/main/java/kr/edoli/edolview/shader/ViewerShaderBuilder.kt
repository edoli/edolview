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
    vec3 v;
    v.r = color_proc(tex.r);
    v.g = color_proc(tex.g);
    v.b = color_proc(tex.b);
    vec3 cm = %colormap_name%_colormap(v);
    """

    // colormap after pixel processing for Mono images
    private val colorProcessMonoColorMap = """
    float v = color_proc(tex.r);
    v = clamp(v, 0.0, 1.0);
    vec3 cm = %colormap_name%_colormap(v);
    """

    private val defaultShader = """
    gl_FragColor = vec4(cm.r, cm.g, cm.b, alpha);
    """

    fun getRGB(colormapName: String) = rgbShaderStore.get(colormapName)
    fun getMono(colormapName: String) = monoShaderStore.get(colormapName)
    fun getCustom(colormapName: String, isMono: Boolean) = build(colormapName, isMono)

    fun clearCache() {
        rgbShaderStore.cleanUp()
        monoShaderStore.cleanUp()
    }

    private fun build(colormapName: String, isMono: Boolean): ExtendedShaderProgram {
        val colormapShaderCode = if (colormapName == "") "" else if (isMono)
            Gdx.files.internal("colormap/mono/${colormapName}.glsl").readString() else
            Gdx.files.internal("colormap/rgb/${colormapName}.glsl").readString()

        val shaderCode = fragShader.replace("%final_shader%", if (customShader != "") customShader else defaultShader)
            .replace("%color_process%", if (isMono) {
                colorProcessMonoColorMap.replace("%colormap_name%", colormapName)
            } else {
                colorProcessRGBColormap.replace("%colormap_name%", colormapName)
            }).replace("%colormap_function%", colormapShaderCode)

        return ExtendedShaderProgram(vertexShader, shaderCode, colormapShaderCode).also {
            if (!it.isCompiled) {
                Platform.showErrorMessage("Error compiling shader[${colormapName}]: " + it.log)
            }
        }
    }
}

