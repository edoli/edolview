package kr.edoli.imview.image

import com.badlogic.gdx.math.Rectangle
import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.core.Rect

/**
 * Created by daniel on 16. 10. 18.
 */
object ImageProc {
    fun getPixel(mat: Mat, x: Int, y: Int): ByteArray {
        val width = mat.cols()
        val height = mat.rows()
        val channels = mat.channels()

        if (x < 0 || x >= width || y < 0 || y >= height) {
            return ByteArray(channels, { 0 })
        }

        val buff = ByteArray(channels)
        mat.get(y, x, buff)

        return buff
    }

    fun crop(mat: Mat, rect: Rect): Mat {
        return mat.submat(rect.y, (rect.y + rect.height),
                rect.x, (rect.x + rect.width))
    }

    fun diff(matA: Mat, matB: Mat, rect: Rect?): Mat {
        val matSubA = if (rect == null) matA else matA.submat(rect)
        val matSubB = if (rect == null) matB else matB.submat(rect)

        val mat = Mat(matSubA.rows(), matSubA.cols(), matSubA.type())
        Core.subtract(matSubA, matSubB, mat)
        return mat
    }
}