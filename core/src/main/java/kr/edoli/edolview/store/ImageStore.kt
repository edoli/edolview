package kr.edoli.edolview.store

import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.image.ImageSpec
import kr.edoli.edolview.image.timesAssign
import org.apache.commons.io.FileUtils
import org.apache.commons.io.FilenameUtils
import org.opencv.core.Mat
import java.io.BufferedReader
import java.io.ByteArrayInputStream
import java.io.File
import java.io.FileReader
import javax.imageio.ImageIO
import kotlin.math.abs

/**
 * Created by daniel on 16. 11. 27.
 */
object ImageStore {

    private const val MAX_MEMORY = 8 * 1024 * 1024 * 1024L

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
        val ext = FilenameUtils.getExtension(path)
        if (!File(path).exists()) {
            return ImageSpec(Mat())
        }
        val file = File(path)
        val bytes = FileUtils.readFileToByteArray(file)

        try {
            if (ext.lowercase() == "flo") {
                val mat = ImageConvert.decodeFlo(bytes)

                return if (mat == null) {
                    ImageSpec(Mat())
                } else {
                    ImageSpec(mat)
                }

            } else {
                val mat = bytesToMat(bytes)

                if (ext.lowercase() == "pfm") {
                    val reader = BufferedReader(FileReader(path))
                    val header = reader.readLine()  // first line
                    val dimension = reader.readLine()  // second line
                    val scale = reader.readLine()  // third line

                    mat *= abs(scale.toDouble())
                }

                return ImageSpec(mat)
            }
        } catch (e: Exception) {
            return ImageSpec(Mat())
        }
    }

    fun bytesToMat(bytes: ByteArray): Mat {
        if (checkMagicNumber(bytes, byteArrayOf(0x49, 0x49, 0x2A, 0x00))) {
            // TIFF
            val img = ImageIO.read(ByteArrayInputStream(bytes))
            return ImageConvert.bufferedToMat(img)
        } else {
            return ImageConvert.decodeBytes(bytes)
        }
    }

    fun checkMagicNumber(bytes: ByteArray, magicNumber: ByteArray): Boolean {
        for (i in magicNumber.indices) {
            if (bytes[i] != magicNumber[i]) {
                return false
            }
        }
        return true
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