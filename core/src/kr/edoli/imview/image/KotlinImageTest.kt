package kr.edoli.imview.image

import com.badlogic.gdx.graphics.Pixmap
import kr.edoli.imview.util.getChannels

/**
 * Created by sjjeon on 16. 12. 20.
 */
object KotlinImageTest {
    fun test(pixmap: Pixmap) : Pixmap {

        val pixels = pixmap.pixels
        val nPixmap = Pixmap(pixmap.width, pixmap.height, pixmap.format)
        val nPixels = nPixmap.pixels

        val width = pixmap.width
        val height = pixmap.height
        val kernel = intArrayOf(-width-1, -width, -width+1,
                -1, 0, 1,
                width-1,width,width+1
                )

        for (tx in 1..width-2) {
            for (ty in 1..height-2) {
                val ind = (tx + ty * width) * 3

                var r = 0
                var g = 0
                var b = 0

                for (cx in -1..1) {
                    for (cy in -1..1) {
                        val pind = ind + (cx + cy * width) * 3
                        r += pixels[pind]
                        g += pixels[pind + 1]
                        b += pixels[pind + 2]
                    }
                }

                nPixels.put(ind, (r / 9).toByte())
                nPixels.put(ind, (g / 9).toByte())
                nPixels.put(ind, (b / 9).toByte())
            }
        }
        return nPixmap
    }
}