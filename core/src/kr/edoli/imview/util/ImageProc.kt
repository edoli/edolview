package kr.edoli.imview.util

import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.math.Rectangle
import kr.edoli.imview.Context

/**
 * Created by daniel on 16. 10. 18.
 */
object ImageProc {
    fun eachPixel(pixmap: Pixmap, processor: (value: Byte, x: Int, y: Int, c: Int) -> Byte) {
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
                    pixels.put(indc, processor(pixels[indc], tx, ty, c))
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

    fun psnr(pixmapA: Pixmap, pixmapB: Pixmap, rect: Rectangle): Double {
        val pixelsA = pixmapA.pixels
        val pixelsB = pixmapB.pixels

        if (pixmapA.width != pixmapB.width || pixmapA.height != pixmapB.height || pixmapA.format != pixmapB.format) {
            return -1.0;
        }

        val x = rect.x.toInt()
        val y = rect.y.toInt()
        val width = rect.width.toInt()
        val height = rect.height.toInt()

        val pixmapWidth = pixmapA.width
        val pixmapHeight = pixmapA.height

        val channels = pixmapA.getChannels()

        val size = width * height * channels

        var mse = 0.0

        for (tx in x..x+width-1) {
            for (ty in y..y+height-1) {
                val ind = (tx + ty * pixmapWidth) * channels

                for (c in 0..channels-1) {
                    val indc = ind + c
                    val value = (pixelsA[indc] - pixelsB[indc]).toByte()

                    mse += value * value
                }
            }
        }

        mse /= size

        return 20 * Math.log10(255.0) - 10 * Math.log10(mse)
    }

    fun psnr(pixmapA: Pixmap, pixmapB: Pixmap): Double {
        val selectBox = Context.selectBox.get()
        if (selectBox.width != 0f && selectBox.height != 0f) {
            return psnr(pixmapA, pixmapB, selectBox)
        }
        return psnr(pixmapA, pixmapB, Rectangle(0f, 0f, pixmapA.width.toFloat(), pixmapA.height.toFloat()))
    }

    fun diff(pixmapA: Pixmap, pixmapB: Pixmap, rect: Rectangle): Pixmap? {
        val pixelsA = pixmapA.pixels
        val pixelsB = pixmapB.pixels

        if (pixmapA.width != pixmapB.width || pixmapA.height != pixmapB.height || pixmapA.format != pixmapB.format) {
            return null;
        }

        val x = rect.x.toInt()
        val y = rect.y.toInt()
        val width = rect.width.toInt()
        val height = rect.height.toInt()

        val pixmapWidth = pixmapA.width
        val pixmapHeight = pixmapA.height

        val channels = pixmapA.getChannels()

        val pixmap = Pixmap(rect.width.toInt(), rect.height.toInt(), pixmapA.format)
        val pixels = pixmap.pixels


        for (tx in x..x+width-1) {
            for (ty in y..y+height-1) {
                val ind = (tx + ty * pixmapWidth) * channels
                val indSub = (tx - x + (ty - y) * width) * channels

                for (c in 0..channels-1) {
                    val indc = ind + c
                    val p1 = pixelsA[indc].toInt() and 0xFF
                    val p2 = pixelsB[indc].toInt() and 0xFF
                    val value = ((p2 - p1) / 2 + 128).toByte()

                    pixels.put(indSub + c, value)
                }
            }
        }

        return pixmap
    }

    fun diff(pixmapA: Pixmap, pixmapB: Pixmap): Pixmap? {
        val selectBox = Context.selectBox.get()
        if (selectBox.width != 0f && selectBox.height != 0f) {
            return diff(pixmapA, pixmapB, selectBox)
        }
        return diff(pixmapA, pixmapB, Rectangle(0f, 0f, pixmapA.width.toFloat(), pixmapA.height.toFloat()))
    }
}