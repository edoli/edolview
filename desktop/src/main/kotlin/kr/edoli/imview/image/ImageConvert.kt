package kr.edoli.imview.image

import org.opencv.core.CvType
import org.opencv.core.Mat
import java.awt.Transparency
import java.awt.image.*


object ImageConvert {
    val tmpMat = Mat()

    fun matToBuffered(mat: Mat): BufferedImage {
        val width = mat.cols()
        val height = mat.rows()
        val channels = mat.channels()
        val bitsPerPixel = mat.bitsPerPixel()
        val arraySize = (mat.total() * channels).toInt()

        when (channels) {
            1 -> mat.convertTo(tmpMat, CvType.CV_8UC1, 255.0)
            3 -> mat.convertTo(tmpMat, CvType.CV_8UC3, 255.0)
            4 -> mat.convertTo(tmpMat, CvType.CV_8UC4, 255.0)
        }

        val rawData = ByteArray(arraySize)
        tmpMat.get(0, 0, rawData)

        val buffer = DataBufferByte(rawData, rawData.size)
        if (channels == 4) {
            val raster = Raster.createInterleavedRaster(buffer, width, height, channels * width, channels, intArrayOf(0, 1, 2, 3), null)
            val colorModel = ComponentColorModel(ColorModel.getRGBdefault().colorSpace, true, false, Transparency.TRANSLUCENT, DataBuffer.TYPE_BYTE)
            return BufferedImage(colorModel, raster, false, null)
        } else {
            val raster = Raster.createInterleavedRaster(buffer, width, height, channels * width, channels, intArrayOf(0, 1, 2), null)
            val colorModel = ComponentColorModel(ColorModel.getRGBdefault().colorSpace, false, true, Transparency.OPAQUE, DataBuffer.TYPE_BYTE)
            return BufferedImage(colorModel, raster, true, null)
        }
    }
}