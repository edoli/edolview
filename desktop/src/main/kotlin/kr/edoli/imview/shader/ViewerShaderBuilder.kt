package kr.edoli.imview.shader

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import org.apache.commons.io.FileUtils
import java.io.File
import javax.swing.JFrame
import javax.swing.JOptionPane

class ViewerShaderBuilder {
    companion object {
        fun getColormapNames(subDir: String): List<String> {
            val file = File("colormap/${subDir}")
            return file.list()?.filter {
                it.contains(".glsl")
            }?.map {
                it.replace(".glsl", "")
            }?.sortedBy { if (it == "color") "" else it } ?: listOf()
        }

        val fragShader: String = Gdx.files.internal("imageShader.frag").readString()
        const val default_pixel_expression = "pow(p * contrast + brightness, 1.0 / gamma)"
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
            FileUtils.readFileToString(File("colormap/mono/${colormapName}.glsl")) else
            FileUtils.readFileToString(File("colormap/rgb/${colormapName}.glsl"))

        val shaderCode = fragShader
                .replace("%color_process%", if (isMono)
                    colorProcessMonoColorMap.replace("%colormap_name%", colormapName) else
                    colorProcessRGBColormap.replace("%colormap_name%", colormapName))
                .replace("%colormap_function%", colormapShaderCode)
                .replace("%extra_code%", extraCode.replace("%pixel_expression%", ""))
                .replace("%pixel_expression%", pixelExpression)

        return ShaderProgram(SpriteShaderBuilder.vertexShader, shaderCode).also {
            if (!it.isCompiled) {
                Thread {
                    val frame = JFrame()
                    frame.isAlwaysOnTop = true
                    JOptionPane.showMessageDialog(frame, "Error compiling shader[${colormapName}]: " + it.log, "Error message",
                            JOptionPane.ERROR_MESSAGE)
                }.start()
            }
        }
    }
}