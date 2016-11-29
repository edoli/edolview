package kr.edoli.imview

import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.math.Rectangle
import kr.edoli.imview.image.QualityMetric

/**
 * Created by sjjeon on 16. 10. 28.
 */
enum class ComparisonMetric {
    PSNR {
        override fun compute(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle) = QualityMetric.psnr(pixmapA, pixmapB, rectangle)
    }, SSIM {
        override fun compute(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle) = QualityMetric.ssim(pixmapA, pixmapB, rectangle)
    }, RMSE {
        override fun compute(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle) = QualityMetric.rmse(pixmapA, pixmapB, rectangle)
    }, MSE {
        override fun compute(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle) = QualityMetric.mse(pixmapA, pixmapB, rectangle)
    };

    companion object {
        val values = values()
    }

    abstract fun compute(pixmapA: Pixmap, pixmapB: Pixmap, rectangle: Rectangle): Double
}