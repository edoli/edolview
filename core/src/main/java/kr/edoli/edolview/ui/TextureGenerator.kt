package kr.edoli.edolview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.GL30
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import com.badlogic.gdx.graphics.glutils.GLVersion
import kr.edoli.edolview.image.split
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc

object TextureGenerator {
    val emptyTexture = Texture(0, 0, Pixmap.Format.RGB888)

    var lastMat: Mat? = null
    var lastVisibleChannel = 0

    fun isChanged(mat: Mat?, visibleChannel: Int): Boolean {
        if (mat == lastMat && visibleChannel == lastVisibleChannel) {
            return false
        }
        return true
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

        when (matVisible.channels()) {
            1 -> matVisible.convertTo(matVisible, CvType.CV_32FC1)
            2 -> matVisible.convertTo(matVisible, CvType.CV_32FC2)
            3 -> matVisible.convertTo(matVisible, CvType.CV_32FC3)
            4 -> matVisible.convertTo(matVisible, CvType.CV_32FC4)
        }

        val numPixels = matVisible.total().toInt()
        val numChannels = matVisible.channels()

        var dataChannels = 4
        if (Gdx.graphics.glVersion.type != GLVersion.Type.OpenGL) {
            Imgproc.cvtColor(matVisible, matVisible, Imgproc.COLOR_RGB2RGBA, 4)
            dataChannels = if (numChannels == 1) 1 else 4
        } else {
            dataChannels = numChannels
        }

        val matData = FloatArray((numPixels * dataChannels))
        matVisible.get(0, 0, matData)


        val internalFormat = when (numChannels) {
            1 -> GL30.GL_RGB32F
            2 -> GL30.GL_RGB32F
            3 -> GL30.GL_RGB32F
            4 -> GL30.GL_RGBA32F
            else -> GL30.GL_RGB32F
        }

        val format = when (numChannels) {
            1 -> GL30.GL_RGB
            2 -> GL30.GL_RGB
            3 -> GL30.GL_RGB
            4 -> GL30.GL_RGBA
            else -> GL30.GL_RGB
        }

        val textureData = FloatTextureData(mat.width(), mat.height(), internalFormat, format, GL30.GL_FLOAT, false)
        textureData.prepare()
        if (numChannels == 1) {
            val buffer = FloatArray(numPixels * 3)
            var i = 0
            matData.forEach { v -> repeat(3) { buffer[i++] = v } }
            textureData.buffer.put(buffer)
        } else if (numChannels == 2) {
            val buffer = FloatArray(numPixels * 3)
            var i = 0
            for (p in 0 until numPixels) {
                val j = p * 2
                buffer[i++] = matData[j]
                buffer[i++] = matData[j + 1]
                buffer[i++] = 0f
            }
            textureData.buffer.put(buffer)
        } else {
            textureData.buffer.put(matData)
        }
        textureData.buffer.position(0)
        return Texture(textureData)
    }
}