package kr.edoli.imview.image

import javafx.scene.image.Image
import javafx.scene.image.PixelFormat
import javafx.scene.image.WritableImage
import javafx.scene.image.WritablePixelFormat
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.imgproc.Imgproc


object ImageConvert {
    val tmpMat = Mat()

    fun matToImage(mat: Mat): Image {
        val width = mat.cols()
        val height = mat.rows()
        val channels = mat.channels()
        val arraySize = (mat.total() * 3).toInt()

        mat.convertTo(tmpMat, CvType.CV_8UC3, 255.0)
        when (channels) {
            1 -> Imgproc.cvtColor(tmpMat, tmpMat, Imgproc.COLOR_GRAY2RGB)
            4 -> Imgproc.cvtColor(tmpMat, tmpMat, Imgproc.COLOR_RGBA2RGB)
        }

        val rawData = ByteArray(arraySize)
        tmpMat.get(0, 0, rawData)

        val wi = WritableImage(width, height)
        val pw = wi.pixelWriter
        pw.setPixels(0, 0, width, height, PixelFormat.getByteRgbInstance(), rawData, 0, width * 3)

        return wi
    }

    fun imageToMat(image: Image): Mat {
        val width = image.width.toInt()
        val height = image.height.toInt()
        val buffer = ByteArray(width * height * 4)

        val reader = image.pixelReader
        val format = WritablePixelFormat.getByteBgraInstance()
        reader.getPixels(0, 0, width, height, format, buffer, 0, width * 4)

        val mat = Mat(height, width, CvType.CV_8UC4)
        mat.put(0, 0, buffer)
        Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGR2RGB)
        return mat
    }
}