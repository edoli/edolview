package kr.edoli.imview.image

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.TextureData
import com.badlogic.gdx.graphics.g2d.Gdx2DPixmap
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.Scalar

/**
 * Created by vclab_sjjeon on 2017-11-14.
 */
object ImageUtils {
    val tmpMat = Mat()

    fun matToPixmap(mat: Mat): Pixmap {

        val width = mat.cols()
        val height = mat.rows()
        val channels = mat.channels()
        val arraySize = (mat.total() * mat.channels()).toInt()

        when (channels) {
            1 -> mat.convertTo(tmpMat, CvType.CV_8UC1, 255.0)
            3 -> mat.convertTo(tmpMat, CvType.CV_8UC3, 255.0)
            4 -> mat.convertTo(tmpMat, CvType.CV_8UC4, 255.0)
        }
        val buff = ByteArray(arraySize)
        tmpMat.get(0, 0, buff)

        val format = when (channels) {
            1 -> Pixmap.Format.LuminanceAlpha
            3 -> Pixmap.Format.RGB888
            4 -> Pixmap.Format.RGBA8888
            else -> Pixmap.Format.RGB888
        }

        val pixmap = Pixmap(width, height, format)
        val pixels = pixmap.pixels

        val q = 255.toByte()

        pixels.rewind()
        if (channels == 1) {
            for (i in 0 until arraySize) {
                pixels.put(i * 2, buff[i])
                pixels.put(i * 2 + 1, q)
            }
        } else {
            for (i in 0 until arraySize) {
                pixels.put(i, buff[i])
            }
        }
        /*
        for (y in 0 until height) {
            for (x in 0 until width) {
                for (c in 0 until channels) {
                    val ind = (y * width + x) * channels + c
                    pixels.put(ind, buff[ind])
                }
            }
        }
        */

        return pixmap
    }
}