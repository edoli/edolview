package kr.edoli.imview

import kr.edoli.imview.image.QualityMetric
import org.opencv.core.Mat
import org.opencv.core.Rect

/**
 * Created by sjjeon on 16. 10. 28.
 */
enum class ComparisonMetric {
    PSNR {
        override fun compute(matA: Mat, matB: Mat, rect: Rect) = QualityMetric.psnr(matA, matB, rect)
    }, SSIM {
        override fun compute(matA: Mat, matB: Mat, rect: Rect) = QualityMetric.ssim(matA, matB, rect)
    }, RMSE {
        override fun compute(matA: Mat, matB: Mat, rect: Rect) = QualityMetric.rmse(matA, matB, rect)
    }, MSE {
        override fun compute(matA: Mat, matB: Mat, rect: Rect) = QualityMetric.mse(matA, matB, rect)
    };

    companion object {
        val values = values()
    }

    abstract fun compute(matA: Mat, matB: Mat, rect: Rect): Double
}