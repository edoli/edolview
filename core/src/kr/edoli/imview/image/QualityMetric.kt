package kr.edoli.imview.image

import org.opencv.core.Core
import org.opencv.core.Mat
import org.opencv.core.Rect

/**
 * Created by daniel on 16. 11. 29.
 */
object QualityMetric {


    fun psnr(matA: Mat, matB: Mat, rect: Rect?): Double {
        val matSubA = if (rect == null) matA else matA.submat(rect)
        val matSubB = if (rect == null) matB else matB.submat(rect)

        return Core.PSNR(matSubA, matSubB)
    }

    fun ssim(matA: Mat, matB: Mat, rect: Rect): Double {

        val mse = mse(matA, matB, rect)

        if (mse == -1.0) {
            return -1.0
        }

        return 20 * Math.log10(255.0) - 10 * Math.log10(mse)
    }


    fun rmse(matA: Mat, matB: Mat, rect: Rect?): Double {

        val mse = mse(matA, matB, rect)

        if (mse == -1.0) {
            return -1.0
        }

        return Math.sqrt(mse)
    }

    fun mse(matA: Mat, matB: Mat, rect: Rect?): Double {
        val matSubA = if (rect == null) matA else matA.submat(rect)
        val matSubB = if (rect == null) matB else matB.submat(rect)


        val matDiff = Mat(matA.rows(), matA.cols(), matA.type())
        Core.absdiff(matA, matB, matDiff)

        val matSquared = Mat(matDiff.rows(), matDiff.cols(), matDiff.type())
        Core.multiply(matDiff, matDiff, matSquared)

        val scalar = Core.sumElems(matSquared)
        val sse = scalar.`val`[0] + scalar.`val`[1] + scalar.`val`[2]

        return sse / (matA.total() * matA.channels())
    }
}