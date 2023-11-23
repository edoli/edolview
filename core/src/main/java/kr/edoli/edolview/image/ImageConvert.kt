package kr.edoli.edolview.image

import com.badlogic.gdx.graphics.Pixmap
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.MatOfByte
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc
import java.awt.Transparency
import java.awt.image.*
import java.nio.*


object ImageConvert {
    val tmpMat = Mat()

    fun matToBuffered(mat: Mat): BufferedImage {
        val width = mat.cols()
        val height = mat.rows()
        val channels = mat.channels()
        val arraySize = (mat.total() * channels).toInt()

        mat.convertTo(tmpMat, CvType.CV_8U, 255.0)

        val rawData = ByteArray(arraySize)
        tmpMat.get(0, 0, rawData)

        return byteArrayToBuffered(rawData, width, height, channels)
    }

    fun bufferedToByteBuffer(bufferedImage: BufferedImage): ByteBuffer {
        val byteBuffer: ByteBuffer
        val dataBuffer = bufferedImage.raster.dataBuffer

        when (dataBuffer.dataType) {
            DataBuffer.TYPE_BYTE -> {
                val pixelData = (dataBuffer as DataBufferByte).data
                byteBuffer = ByteBuffer.allocateDirect(pixelData.size)
                byteBuffer.order(ByteOrder.LITTLE_ENDIAN)
                byteBuffer.put(pixelData)
            }
            DataBuffer.TYPE_USHORT -> {
                val pixelData = (dataBuffer as DataBufferUShort).data
                byteBuffer = ByteBuffer.allocateDirect(pixelData.size * 2)
                byteBuffer.order(ByteOrder.LITTLE_ENDIAN)
                byteBuffer.asShortBuffer().put(ShortBuffer.wrap(pixelData))
            }
            DataBuffer.TYPE_SHORT -> {
                val pixelData = (dataBuffer as DataBufferShort).data
                byteBuffer = ByteBuffer.allocateDirect(pixelData.size * 2)
                byteBuffer.order(ByteOrder.LITTLE_ENDIAN)
                byteBuffer.asShortBuffer().put(ShortBuffer.wrap(pixelData))
            }
            DataBuffer.TYPE_INT -> {
                val pixelData = (dataBuffer as DataBufferInt).data
                byteBuffer = ByteBuffer.allocateDirect(pixelData.size * 4)
                byteBuffer.order(ByteOrder.LITTLE_ENDIAN)
                byteBuffer.asIntBuffer().put(IntBuffer.wrap(pixelData))
            }
            DataBuffer.TYPE_FLOAT -> {
                val pixelData = (dataBuffer as DataBufferFloat).data
                byteBuffer = ByteBuffer.allocateDirect(pixelData.size * 4)
                byteBuffer.order(ByteOrder.LITTLE_ENDIAN)
                byteBuffer.asFloatBuffer().put(FloatBuffer.wrap(pixelData))
            }
            DataBuffer.TYPE_DOUBLE -> {
                val pixelData = (dataBuffer as DataBufferDouble).data
                byteBuffer = ByteBuffer.allocateDirect(pixelData.size * 8)
                byteBuffer.order(ByteOrder.LITTLE_ENDIAN)
                byteBuffer.asDoubleBuffer().put(DoubleBuffer.wrap(pixelData))
            }
            else -> {
                throw IllegalArgumentException("Not implemented for data buffer type: " + dataBuffer::javaClass)
            }

        }

        return byteBuffer
    }

    fun bufferedToMat(bufferedImage: BufferedImage): Mat {
        val width = bufferedImage.width
        val height = bufferedImage.height
        val channels = bufferedImage.colorModel.numComponents

        val dataType = bufferedImage.data.dataBuffer.dataType
        val rawData = bufferedToByteBuffer(bufferedImage)
        val mat: Mat

        when (dataType) {
            DataBuffer.TYPE_BYTE -> {
                val type = CvType.makeType(0, channels)
                mat = Mat(height, width, type, rawData)
            }
            DataBuffer.TYPE_USHORT -> {
                val type = CvType.makeType(7, channels)
                mat = Mat(height, width, type, rawData)
            }
            DataBuffer.TYPE_SHORT -> {
                val type = CvType.makeType(3, channels)
                mat = Mat(height, width, type, rawData)
            }
            DataBuffer.TYPE_INT -> {
                val type = CvType.CV_8UC4
                mat = Mat(height, width, type, rawData)
            }
            DataBuffer.TYPE_FLOAT -> {
                val type = CvType.makeType(5, channels)
                mat = Mat(height, width, type, rawData)
            }
            DataBuffer.TYPE_DOUBLE -> {
                val type = CvType.makeType(6, channels)
                mat = Mat(height, width, type, rawData)
            }
            else -> {
                val type = CvType.makeType(0, channels)
                mat = Mat(height, width, type, rawData)
            }
        }

        return arrangeChannels(mat, bufferedImage.type)
    }


    private fun arrangeChannels(mat: Mat, bufferedImageType: Int): Mat {
        when (mat.channels()) {
            4 -> {
                val matChannels = mat.split()
                when (bufferedImageType) {
                    in arrayOf(1) -> {
                        Core.merge(listOf(matChannels[1], matChannels[2], matChannels[3]), mat)
                    }
                    in arrayOf(2, 3) -> {
                        Core.merge(listOf(matChannels[1], matChannels[2], matChannels[3], matChannels[0]), mat)
                    }
                    in arrayOf(4) -> {
                        Core.merge(listOf(matChannels[3], matChannels[2], matChannels[1]), mat)
                    }
                    in arrayOf(6, 7) -> {
                        Core.merge(listOf(matChannels[3], matChannels[2], matChannels[1], matChannels[0]), mat)
                    }
                }
            }
            3 -> {
                val matChannels = mat.split()
                when (bufferedImageType) {
                    in arrayOf(5) -> {
                        Core.merge(listOf(matChannels[2], matChannels[1], matChannels[0]), mat)
                    }
                }
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

    fun decodeBytes(bytes: ByteArray): Mat {
        val matBytes = MatOfByte(*bytes)
        val mat = Imgcodecs.imdecode(matBytes, -1)

        when (mat.channels()) {
            3 -> {
                Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGR2RGB)
            }
            4 -> {
                Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGRA2RGBA)
            }
        }

        return mat
    }

    fun decodeFlo(bytes: ByteArray): Mat? {
        val buffer = ByteBuffer.wrap(bytes, 0, 12).order(ByteOrder.LITTLE_ENDIAN)
        val magic = buffer.getFloat(0)

        if (202021.25f != magic) {
            return null
        }

        val width = buffer.getInt(4)
        val height = buffer.getInt(8)

        val mat = Mat(height, width, CvType.CV_32FC2)
        val numPixels = 2 * width * height
        val floatArray = FloatArray(numPixels)
        ByteBuffer.wrap(bytes, 12, numPixels * 4).order(ByteOrder.LITTLE_ENDIAN).asFloatBuffer().get(floatArray)

        mat.put(0, 0, floatArray)
        return mat
    }
}