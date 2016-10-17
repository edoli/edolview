package kr.edoli.imview.util

import com.badlogic.gdx.graphics.Pixmap

/**
 * Created by daniel on 16. 10. 18.
 */
object ImageProc {
    fun eachPixel(pixmap: Pixmap, processor: (value: Byte, x: Int, y: Int, c: Int) -> Unit) {
        val pixels = pixmap.pixels

        val width = pixmap.width
        val height = pixmap.height

        val format = pixmap.format
        val channels = if (format == Pixmap.Format.RGBA4444 || format == Pixmap.Format.RGBA8888) 4
            else if (format == Pixmap.Format.RGB565 || format == Pixmap.Format.RGB888) 3
            else 1


        val size = width * height * channels

        var mse = 0.0

        for (tx in 0..width-1) {
            for (ty in 0..height-1) {
                val ind = (tx + ty * width) * channels

                for (c in 0..channels-1) {
                    val indc = ind + c
                    processor(pixels[indc], tx, ty, c)
                }
            }
        }
    }

    fun getPixel(pixmap: Pixmap, x: Int, y: Int): ByteArray {
        val pixels = pixmap.pixels

        val width = pixmap.width
        val height = pixmap.height

        val format = pixmap.format
        val channels = if (format == Pixmap.Format.RGBA4444 || format == Pixmap.Format.RGBA8888) 4
            else if (format == Pixmap.Format.RGB565 || format == Pixmap.Format.RGB888) 3
            else 1

        if (x < 0 || x >= width || y < 0 || y >= height) {
            return ByteArray(channels, { 0 })
        }

        if (channels == 1) {
            return byteArrayOf(pixels[(x + y * width)])
        } else if (channels == 3) {
            val ind = (x + y * width) * channels
            return byteArrayOf(pixels[ind], pixels[ind + 1], pixels[ind + 2])
        } else if (channels == 4) {
            val ind = (x + y * width) * channels
            return byteArrayOf(pixels[ind], pixels[ind + 1], pixels[ind + 2], pixels[ind + 3])
        }
        return byteArrayOf(pixels[(x + y * width)])
    }
}