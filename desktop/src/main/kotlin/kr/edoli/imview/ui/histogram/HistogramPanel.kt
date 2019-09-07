package kr.edoli.imview.ui

import kr.edoli.imview.ImContext
import kr.edoli.imview.util.Histogram
import org.opencv.core.Core

class HistogramPanel : Panel(false) {
    val histograms = ArrayList<Histogram>()

    init {
        ImContext.mainImage.subscribe { mat ->
            if (mat == null) return@subscribe


        }
    }
}