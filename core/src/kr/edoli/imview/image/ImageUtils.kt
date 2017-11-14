package kr.edoli.imview.image

import com.badlogic.gdx.graphics.Pixmap
import org.opencv.core.Mat

/**
 * Created by vclab_sjjeon on 2017-11-14.
 */
object ImageUtils {
    fun matToPixmap(mat: Mat): Pixmap {
        val buff = ByteArray((mat.total() * mat.channels()).toInt())
        mat.get(0, 0, buff)

        val width = mat.cols()
        val height = mat.rows()
        val channels = mat.channels()

        val pixmap = Pixmap(width, height, Pixmap.Format.RGB888)
        val pixels = pixmap.pixels

        for (tx in 0 until width) {
            for (ty in 0 until height) {
                val ind = (tx + ty * width) * channels

                for (c in 0 until channels) {
                    val byte = buff[ind + c]
                    pixels.put(ind + c, byte)
                }
            }
        }

        return pixmap
    }
}