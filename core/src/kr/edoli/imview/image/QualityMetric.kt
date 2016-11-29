package kr.edoli.imview.image

import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.math.Rectangle
import kr.edoli.imview.util.getChannels

/**
 * Created by daniel on 16. 11. 29.
 */
object QualityMetric {


    fun psnr(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle?): Double {

        val mse = mse(pixmapA, pixmapB, rectangle)

        if (mse == -1.0) {
            return -1.0
        }

        return 20 * Math.log10(255.0) - 10 * Math.log10(mse)
    }

    fun ssim(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle?): Double {

        val mse = mse(pixmapA, pixmapB, rectangle)

        if (mse == -1.0) {
            return -1.0
        }

        return 20 * Math.log10(255.0) - 10 * Math.log10(mse)
    }


    fun rmse(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle?): Double {

        val mse = mse(pixmapA, pixmapB, rectangle)

        if (mse == -1.0) {
            return -1.0
        }

        return Math.sqrt(mse)
    }

    fun mse(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle?): Double {
        val rect = if (rectangle != null && rectangle.width != 0f && rectangle.height != 0f) rectangle
        else Rectangle(0f, 0f, pixmapA.width.toFloat(), pixmapA.height.toFloat())

        val start = System.nanoTime()

        val pixelsA = pixmapA.pixels
        val pixelsB = pixmapB.pixels

        if (pixmapA.width != pixmapB.width || pixmapA.height != pixmapB.height || pixmapA.format != pixmapB.format) {
            return -1.0
        }

        val x = rect.x.toInt()
        val y = rect.y.toInt()
        val width = rect.width.toInt()
        val height = rect.height.toInt()

        val pixmapWidth = pixmapA.width

        val channels = pixmapA.getChannels()

        val size = width * height * channels

        var mse = 0.0

        for (tx in x..x+width-1) {
            for (ty in y..y+height-1) {
                val ind = (tx + ty * pixmapWidth) * channels

                for (c in 0..channels-1) {
                    val indc = ind + c
                    val p1 = pixelsA[indc].toInt() and 0xFF
                    val p2 = pixelsB[indc].toInt() and 0xFF
                    val value = p2 - p1

                    mse += value * value
                }
            }
        }

        mse /= size

        println(System.nanoTime() - start)

        return mse
    }
}