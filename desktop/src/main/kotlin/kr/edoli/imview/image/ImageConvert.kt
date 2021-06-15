package kr.edoli.imview.image

import com.badlogic.gdx.graphics.Pixmap
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc
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

        return if (channels == 4) {
            val raster = Raster.createInterleavedRaster(buffer, width, height, channels * width, channels, intArrayOf(0, 1, 2, 3), null)
            val colorModel = ComponentColorModel(ColorModel.getRGBdefault().colorSpace, true, false, Transparency.TRANSLUCENT, DataBuffer.TYPE_BYTE)
            BufferedImage(colorModel, raster, false, null)
        } else {
            val raster = Raster.createInterleavedRaster(buffer, width, height, channels * width, channels, intArrayOf(0, 1, 2), null)
            val colorModel = ComponentColorModel(ColorModel.getRGBdefault().colorSpace, false, true, Transparency.OPAQUE, DataBuffer.TYPE_BYTE)
            BufferedImage(colorModel, raster, true, null)
        }
    }

    fun bufferedToMat(bufferedImage: BufferedImage): Mat {
        val width = bufferedImage.width
        val height = bufferedImage.height
        val channels = if (bufferedImage.colorModel.hasAlpha()) 4 else 3

        val type = if (channels == 4) CvType.CV_8UC4 else CvType.CV_8UC3

        val dataBuffer = bufferedImage.data.dataBuffer as DataBufferByte
        val rawData = dataBuffer.data
        val mat = Mat(height, width, type)
        mat.put(0, 0, rawData)

        when (channels) {
            4 -> {
                val matChannels = mat.split()
                Core.merge(listOf(matChannels[3], matChannels[2], matChannels[1], matChannels[0]), mat)
            }
        }
        return mat
    }

    fun pixmapToBuffered(pixmap: Pixmap): BufferedImage {
        val channels = 4
        val width = pixmap.width
        val height = pixmap.height
        val rawData = pixmap.pixels.array()
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

    fun byteArrayToBuffered(byteArray: ByteArray, width: Int, height: Int, channels: Int): BufferedImage {
        val buffer = DataBufferByte(byteArray, byteArray.size)

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