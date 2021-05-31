package kr.edoli.imview.store

import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import kr.edoli.imview.image.ImageSpec
import kr.edoli.imview.image.bitsPerPixel
import kr.edoli.imview.image.timesAssign
import kr.edoli.imview.image.typeMax
import org.apache.commons.io.FileUtils
import org.apache.commons.io.FilenameUtils
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.core.MatOfByte
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc
import java.io.BufferedReader
import java.io.File
import java.io.FileReader
import java.util.concurrent.TimeUnit
import kotlin.math.abs

/**
 * Created by daniel on 16. 11. 27.
 */
object ImageStore {

    private const val MAX_MEMORY = 8 * 1024 * 1024 * 1024L

    private val imageStoreMap = CacheBuilder.newBuilder()
            .maximumWeight(MAX_MEMORY)
            .expireAfterAccess(5, TimeUnit.MINUTES)
            .weigher { _: ImageDesc, v1: ImageSpec -> (v1.mat.total() * v1.mat.channels()).toInt() }
            .removalListener<ImageDesc, ImageSpec> { it.value?.mat?.release() }
            .build(object : CacheLoader<ImageDesc, ImageSpec>() {
                override fun load(key: ImageDesc): ImageSpec {
                    return loadFromPath(key.path)
                }
            })

    private fun loadFromPath(path: String): ImageSpec {
        val ext = FilenameUtils.getExtension(path)
        if (!File(path).exists()) {
            return ImageSpec(Mat(), 0.0, 0, 0 )
        }
        val bytes = FileUtils.readFileToByteArray(File(path))
        val mat = Imgcodecs.imdecode(MatOfByte(*bytes), -1)

        when (mat.channels()) {
            3 -> {
                Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGR2RGB)
            }
            4 -> {
                Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGRA2RGBA)
            }
        }

        if (ext.toLowerCase() == "pfm") {
            val reader = BufferedReader(FileReader(path))
            val header = reader.readLine()
            val dimension = reader.readLine()
            val scale = reader.readLine()

            mat *= abs(scale.toDouble())
        }

        return ImageSpec(mat, mat.typeMax(), mat.channels(), mat.bitsPerPixel())
    }

    fun clearCache() {
        imageStoreMap.cleanUp()
        imageStoreMap.invalidateAll()
    }

    fun normalize(mat: Mat): Mat {
        when (mat.channels()) {
            1 -> {
                val alpha = when (mat.type()) {
                    CvType.CV_8U -> 1.0 / 255.0
                    CvType.CV_16U -> 1.0 / 65535.0
                    else -> 1.0
                }
                mat.convertTo(mat, CvType.CV_64FC3, alpha)
            }
            3 -> {
                when {
                    mat.type() == CvType.CV_8UC3 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 255.0)
                    mat.type() == CvType.CV_16UC3 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 65535.0)
                    else -> mat.convertTo(mat, CvType.CV_64FC3)
                }
            }
            4 -> {
                when {
                    mat.type() == CvType.CV_8UC4 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 255.0)
                    mat.type() == CvType.CV_16UC4 -> mat.convertTo(mat, CvType.CV_64FC3, 1.0 / 65535.0)
                    else -> mat.convertTo(mat, CvType.CV_64FC3)
                }
            }
        }
        return mat
    }

    fun get(file: File): ImageSpec {
        return get(file.absolutePath, file.lastModified())
    }

    fun get(path: String, lastModified: Long): ImageSpec {
        val processedName = process(path)
        return imageStoreMap[ImageDesc(processedName, lastModified)]
    }

    private fun process(name: String): String {
        var pName = name
        while (pName.contains("//")) {
            pName = pName.replace("//", "/")
        }
        return pName
    }

    data class ImageDesc(val path: String, val lastModified: Long)
}