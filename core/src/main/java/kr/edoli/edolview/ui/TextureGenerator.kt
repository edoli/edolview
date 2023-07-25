package kr.edoli.edolview.ui

import com.badlogic.gdx.graphics.GL30
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import kr.edoli.edolview.image.nbytes
import kr.edoli.edolview.image.split
import kr.edoli.edolview.util.NativeBuffer
import org.opencv.core.CvType
import org.opencv.core.Mat

object TextureGenerator {
    val emptyTexture = Texture(0, 0, Pixmap.Format.RGB888)

    var lastMat: Mat? = null
    var lastVisibleChannel = 0

    fun isChanged(mat: Mat?, visibleChannel: Int): Boolean {
        return !(mat == lastMat && visibleChannel == lastVisibleChannel)
    }

    fun load(mat: Mat?, visibleChannel: Int): Texture {
        lastMat = mat
        lastVisibleChannel = visibleChannel

        if (mat == null) {
            return emptyTexture
        }

        val matVisible = if (visibleChannel == 0) {
            mat.clone()
        } else {
            val matChannels = mat.split()
            matChannels[visibleChannel - 1].clone()
        }

        // float32 for OpenGL buffer
        matVisible.convertTo(matVisible, CvType.CV_32F)

        val numChannels = matVisible.channels()

        val internalFormat = when (numChannels) {
            1 -> GL30.GL_R32F
            2 -> GL30.GL_RG32F
            3 -> GL30.GL_RGB32F
            4 -> GL30.GL_RGBA32F
            else -> GL30.GL_RGB32F
        }

        val format = when (numChannels) {
            1 -> GL30.GL_RED
            2 -> GL30.GL_RG
            3 -> GL30.GL_RGB
            4 -> GL30.GL_RGBA
            else -> GL30.GL_RGB
        }

        val textureData = FloatTextureData(mat.width(), mat.height(), internalFormat, format, GL30.GL_FLOAT, false)
        textureData.prepare()

        NativeBuffer.wrapAddress(textureData.buffer, matVisible.dataAddr(), matVisible.nbytes)

        textureData.buffer.position(0)
        return Texture(textureData)
    }
}