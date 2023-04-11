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
        const val default_pixel_expression = "pow(p * pow(2.0, exposure) + brightness, 1.0 / gamma)"
    }

    private val rgbShaderStore = CacheBuilder.newBuilder()
            .removalListener<String, ShaderProgram> { it.value?.dispose() }
            .build(object : CacheLoader<String, ShaderProgram>() {
                override fun load(shaderName: String): ShaderProgram {
                    return build(shaderName, false)
                }
            })

    private val monoShaderStore = CacheBuilder.newBuilder()
            .removalListener<String, ShaderProgram> { it.value?.dispose() }
            .build(object : CacheLoader<String, ShaderProgram>() {
                override fun load(shaderName: String): ShaderProgram {
                    return build(shaderName, true)
                }
            })

    var extraCode = ""
    var pixelExpression = default_pixel_expression

    // colormap before pixel processing for RGB images
    private val colorProcessRGBColormap = """
    vec3 t = %colormap_name%_colormap(tex.rgb);
    vec3 v;
    p = t.r;
    v.r = %pixel_expression%;
    p = t.g;
    v.g = %pixel_expression%;
    p = t.b;
    v.b = %pixel_expression%;
    gl_FragColor = v_color * vec4(v, 1.0);
    gl_FragColor.a = alpha;
    """

    // colormap after pixel processing for Mono images
    private val colorProcessMonoColorMap = """
    p = tex.r;
    float v = %pixel_expression%;
    v = clamp(v, 0.0, 1.0);
    vec3 color = %colormap_name%_colormap(v);
    gl_FragColor = vec4(color.r, color.g, color.b, alpha);
    """

    fun getRGB(colormapName: String) = rgbShaderStore.get(colormapName)
    fun getMono(colormapName: String) = monoShaderStore.get(colormapName)

    fun clearCache() {
        rgbShaderStore.cleanUp()
        monoShaderStore.cleanUp()
    }

    private fun build(colormapName: String, isMono: Boolean): ShaderProgram {
        val colormapShaderCode = if (isMono)
            Gdx.files.internal("colormap/mono/${colormapName}.glsl").readString() else
            Gdx.files.internal("colormap/rgb/${colormapName}.glsl").readString()

        val shaderCode = fragShader
                .replace("%color_process%", if (isMono)
                    colorProcessMonoColorMap.replace("%colormap_name%", colormapName) else
                    colorProcessRGBColormap.replace("%colormap_name%", colormapName))
                .replace("%colormap_function%", colormapShaderCode)
                .replace("%extra_code%", extraCode.replace("%pixel_expression%", ""))
                .replace("%pixel_expression%", pixelExpression)

        return ShaderProgram(vertexShader, shaderCode).also {
            if (!it.isCompiled) {
                Platform.showErrorMessage("Error compiling shader[${colormapName}]: " + it.log)
            }
        }
    }
}

