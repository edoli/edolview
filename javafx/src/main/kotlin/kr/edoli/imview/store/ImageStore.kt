package kr.edoli.imview.store

import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import kr.edoli.imview.image.ImageSpec
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc
import java.io.File

/**
 * Created by daniel on 16. 11. 27.
 */
object ImageStore {

    private const val MAX_MEMORY = 1024 * 1024 * 1024L

    private val imageStoreMap = CacheBuilder.newBuilder()
            .maximumWeight(MAX_MEMORY)
            .weigher { _: ImageDesc, v1: ImageSpec -> (v1.mat.total() * v1.mat.channels()).toInt() }
            .removalListener<ImageDesc, ImageSpec> { it.value?.mat?.release() }
            .build(object : CacheLoader<ImageDesc, ImageSpec>() {
                override fun load(key: ImageDesc): ImageSpec {
                    return loadFromPath(key.path)
                }
            })

    private fun loadFromPath(path: String): ImageSpec {
        val mat = Imgcodecs.imread(path, -1)

        when (mat.channels()) {
            3 -> {
                Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGR2RGB)
            }
            4 -> {
                Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGRA2RGBA)
            }
        }

        return ImageSpec(mat, getMaxValue(mat), mat.channels(), ImageSpec.bitsPerPixel(mat))
    }

    fun clearCache() {
        imageStoreMap.invalidateAll()
    }

    private fun getMaxValue(mat: Mat): Double {
        when (mat.type()) {
            CvType.CV_8U -> return 255.0
            CvType.CV_16U -> return 65535.0
            CvType.CV_8UC3 -> return 255.0
            CvType.CV_16UC3 -> return 65535.0
            CvType.CV_8UC4 -> return 255.0
            CvType.CV_16UC4 -> return 65535.0
        }
        return -1.0
    }

    fun normalize(mat: Mat): Mat {
        when (mat.channels()) {
            1 -> {
                val alpha = when (mat.type()) {
                    CvType.CV_8U -> 1.0 / 255.0
                    CvType.CV_16U -> 1.0 / 65535.0
                    else -> 1.0
                }
                mat.convertTo(mat, CvType.CV_32F, alpha)
            }
            3 -> {
                if (mat.type() == CvType.CV_8UC3) {
                    mat.convertTo(mat, CvType.CV_32FC3, 1.0 / 255.0)
                }
                if (mat.type() == CvType.CV_16UC3) {
                    mat.convertTo(mat, CvType.CV_32FC3, 1.0 / 65535.0)
                }
            }
            4 -> {
                if (mat.type() == CvType.CV_8UC4) {
                    mat.convertTo(mat, CvType.CV_32FC4, 1.0 / 255.0)
                }
                if (mat.type() == CvType.CV_16UC4) {
                    mat.convertTo(mat, CvType.CV_32FC4, 1.0 / 65535.0)
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

    /*
    fun put(path: String, mat: Mat) {
        val desc = ImageDesc(Where.None, path)
        imageStoreMap[desc]?.release()
        imageStoreMap.put(desc, mat)
    }
    */

    private fun process(name: String): String {
        var pName = name
        while (pName.contains("//")) {
            pName = pName.replace("//", "/")
        }
        return pName
    }

    data class ImageDesc(val path: String, val lastModified: Long)
}