package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.GL30
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.glutils.FloatTextureData
import kr.edoli.imview.image.split
import org.opencv.core.CvType
import org.opencv.core.Mat

object TextureGenerator {
    val emptyTexture = Texture(0, 0, Pixmap.Format.RGB888)

    var lastMat: Mat? = null
    var lastImageChannels = BooleanArray(500) { false }

    fun isChanged(mat: Mat?, imageChannels: BooleanArray): Boolean {
        val checkImageChannels = lastImageChannels.zip(imageChannels)
        val validateImageChannels = checkImageChannels.fold(true) { acc, pair ->
            (pair.first == pair.second) && acc
        }
        if (mat == lastMat && validateImageChannels) {
            return false
        }
        return true
    }

    fun load(mat: Mat?, imageChannels: BooleanArray): Texture {
        lastMat = mat
        lastImageChannels = imageChannels

        if (mat == null) {
            return emptyTexture
        }

        val showAllChannels = imageChannels[0]
        val visibleChannels = imageChannels.let { it.sliceArray(1 until it.size) }
        val firstVisibleChannel = visibleChannels.indexOfFirst { it }

        val matVisible = if (showAllChannels) {
            mat.clone()
        } else {
            val matChannels = mat.split()
            matChannels[firstVisibleChannel].clone()
        }

        when (matVisible.channels()) {
            1 -> matVisible.convertTo(matVisible, CvType.CV_32FC1)
            2 -> matVisible.convertTo(matVisible, CvType.CV_32FC2)
            3 -> matVisible.convertTo(matVisible, CvType.CV_32FC3)
            4 -> matVisible.convertTo(matVisible, CvType.CV_32FC4)
        }

        val numChannels = matVisible.channels()
        val matData = FloatArray((matVisible.total() * numChannels).toInt())
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
            val buffer = FloatArray(matData.size * 3)
            var i = 0
            matData.forEach { v -> repeat(3) { buffer[i++] = v } }
            textureData.buffer.put(buffer)
        } else {
            textureData.buffer.put(matData)
        }
        textureData.buffer.position(0)
        return Texture(textureData)
    }
}