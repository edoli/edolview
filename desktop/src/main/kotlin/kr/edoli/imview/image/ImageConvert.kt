package kr.edoli.imview.image

import com.badlogic.gdx.graphics.Pixmap
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc
import java.awt.Transparency
import java.awt.image.*
import java.nio.ByteBuffer


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

        return byteArrayToBuffered(rawData, width, height, channels)
    }

    fun bufferedToMat(bufferedImage: BufferedImage): Mat {
        val width = bufferedImage.width
        val height = bufferedImage.height
        val channels = if (bufferedImage.colorModel.hasAlpha()) 4 else 3


        val dataBuffer = bufferedImage.data.dataBuffer

        val mat: Mat
        when (dataBuffer.dataType) {
            DataBuffer.TYPE_INT -> {
                val dataBufferTyped = bufferedImage.data.dataBuffer as DataBufferInt
                val intData = dataBufferTyped.data
                val byteBuffer = ByteBuffer.allocate(intData.size * 4)
                val intBuffer = byteBuffer.asIntBuffer()
                intBuffer.put(intData)
                val rawData = byteBuffer.array()
                val type = CvType.CV_8UC4
                mat = Mat(height, width, type)
                mat.put(0, 0, rawData)
            }
            DataBuffer.TYPE_BYTE -> {
                val dataBufferTyped = bufferedImage.data.dataBuffer as DataBufferByte
                val byteData = dataBufferTyped.data
                val byteBuffer = ByteBuffer.allocate(byteData.size * channels)
                byteBuffer.put(byteData)
                val rawData = byteBuffer.array()
                val type = CvType.CV_8UC4
                mat = Mat(height, width, type)
                mat.put(0, 0, rawData)
            }
            else -> {
                val dataBufferTyped = bufferedImage.data.dataBuffer as DataBufferByte
                val rawData = dataBufferTyped.data
                val type = if (channels == 4) CvType.CV_8UC4 else CvType.CV_8UC3
                mat = Mat(height, width, type)
                mat.put(0, 0, rawData)
            }
        }

        return flipMatChannels(mat)
    }

    private fun flipMatChannels(mat: Mat): Mat {
        when (mat.channels()) {
            4 -> {
                val matChannels = mat.split()
                Core.merge(listOf(matChannels[1], matChannels[2], matChannels[3], matChannels[0]), mat)
            }
        }
        return mat
    }

    fun pixmapToBuffered(pixmap: Pixmap): BufferedImage {
        val channels = 4
        val width = pixmap.width
        val height = pixmap.height
        val rawData = pixmap.pixels.array()
        return byteArrayToBuffered(rawData, width, height, channels)
    }

    fun byteArrayToBuffered(byteArray: ByteArray, width: Int, height: Int, channels: Int): BufferedImage {
        val buffer = DataBufferByte(byteArray, byteArray.size)

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
}