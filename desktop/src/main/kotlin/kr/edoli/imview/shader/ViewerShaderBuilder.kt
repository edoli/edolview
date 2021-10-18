package kr.edoli.imview.shader

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import kr.edoli.imview.image.ImageSpec
import kr.edoli.imview.store.ImageStore
import org.apache.commons.io.FileUtils
import java.io.File
import java.util.concurrent.TimeUnit

class ViewerShaderBuilder {
    companion object {
        fun getColormapNames(): List<String> {
            val file = File("colormap")
            return file.list()?.filter { it.contains(".glsl")
            }?.map {
                it.replace(".glsl", "")
            }?.sortedBy { if (it == "normal") "" else it } ?: listOf()
        }

        val fragShader: String = Gdx.files.internal("imageShader.frag").readString()
        const val default_pixel_expression = "pow(p * contrast + brightness, 1.0 / gamma)"
    }

    private val shaderStore = CacheBuilder.newBuilder()
            .removalListener<String, ShaderProgram> { it.value?.dispose() }
            .build(object : CacheLoader<String, ShaderProgram>() {
                override fun load(shaderName: String): ShaderProgram {
                    return build(shaderName)
                }
            })

    var extraCode = ""
    var pixelExpression = default_pixel_expression

    private val colorProcessDefault = """
    vec4 v;
    p = tex.r;
    v.r = %pixel_expression%;
    p = tex.g;
    v.g = %pixel_expression%;
    p = tex.b;
    v.b = %pixel_expression%;
    gl_FragColor = v_color * v;
    gl_FragColor.a = alpha;
    """

    private val colorProcessUseColorMap = """
    p = tex.r;
    float v = %pixel_expression%;
    v = clamp(v, 0.0, 1.0);
    vec3 color = %colormap_name%_colormap(v);
    gl_FragColor = vec4(color.r, color.g, color.b, alpha);
    """

    private val defaultColorShader = build()

    fun get(colormapName: String? = null): ShaderProgram {
        return if (colormapName == null) {
            defaultColorShader
        } else {
            shaderStore.get(colormapName)
        }
    }

    fun clearCache() {
        shaderStore.cleanUp()
    }

    private fun build(colormapName: String? = null): ShaderProgram {
        val colormapShaderCode =
                if (colormapName == null) ""
                else FileUtils.readFileToString(File("colormap/${colormapName}.glsl"))

        val shaderCode = fragShader
                .replace("%color_process%", if (colormapName == null) colorProcessDefault
                    else colorProcessUseColorMap.replace("%colormap_name%", colormapName))
                .replace("%colormap_function%", colormapShaderCode)
                .replace("%extra_code%", extraCode.replace("%pixel_expression%", ""))
                .replace("%pixel_expression%", pixelExpression)

        return ShaderProgram(SpriteShaderBuilder.vertexShader, shaderCode).also {
            require(it.isCompiled) { "Error compiling shader[${colormapName}]: " + it.log }
        }
    }
}