package kr.edoli.imview.store

import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import kr.edoli.imview.image.ImageConvert
import kr.edoli.imview.image.ImageSpec
import kr.edoli.imview.image.timesAssign
import org.apache.commons.io.FileUtils
import org.apache.commons.io.FilenameUtils
import org.opencv.core.Mat
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
            return ImageSpec(Mat())
        }
        val bytes = FileUtils.readFileToByteArray(File(path))
        val mat = ImageConvert.bytesToMat(bytes)

        if (ext.lowercase() == "pfm") {
            val reader = BufferedReader(FileReader(path))
            // val header = reader.readLine()
            // val dimension = reader.readLine()
            val scale = reader.readLine()

            mat *= abs(scale.toDouble())
        }

        return ImageSpec(mat)
    }

    fun clearCache() {
        imageStoreMap.cleanUp()
        imageStoreMap.invalidateAll()
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