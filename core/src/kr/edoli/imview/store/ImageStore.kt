package kr.edoli.imview.store

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.files.FileHandle
import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader
import kr.edoli.imview.image.ImageProc
import org.opencv.core.Core
import org.opencv.core.CvType
import org.opencv.core.Mat
import org.opencv.highgui.Highgui
import org.opencv.imgcodecs.Imgcodecs
import org.opencv.imgproc.Imgproc

/**
 * Created by daniel on 16. 11. 27.
 */
object ImageStore {

    private val MAX_MEMORY = 1024 * 1024 * 1024L

    private val imageStoreMap = CacheBuilder.newBuilder()
            .maximumWeight(MAX_MEMORY)
            .weigher { k1: ImageDesc, v1: Mat? -> if (v1 == null) 0 else (v1.total() * v1.channels()).toInt() }
            .removalListener<ImageDesc, Mat?>{ it.value?.release() }
            .build(object : CacheLoader<ImageDesc, Mat?>() {
                override fun load(key: ImageDesc): Mat? {
                    when (key.where) {
                        Where.Absolute -> return loadFromPath(Gdx.files.absolute(key.name))
                        Where.Internal -> return loadFromPath(Gdx.files.internal(key.name))
                        Where.External -> return loadFromPath(Gdx.files.external(key.name))
                    }
                    return null
                }
            })

    fun loadFromPath(fileHandle: FileHandle): Mat? {
        val mat = Imgcodecs.imread(fileHandle.file().absolutePath)

        if (mat.channels() == 3) {
            Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGR2RGB)
        }
        if (mat.channels() == 4) {
            Imgproc.cvtColor(mat, mat, Imgproc.COLOR_BGRA2RGBA)
        }
        return mat
    }

    fun get(where: Where, name: String): Mat? {
        var processedName = name
        while (processedName.contains("//")) {
            processedName = processedName.replace("//", "/")
        }
        return imageStoreMap[ImageDesc(where, name)]
    }

    fun put(name: String, mat: Mat) {
        val desc = ImageDesc(Where.None, name)
        imageStoreMap[desc]?.release()
        imageStoreMap.put(desc, mat)
    }

    enum class Where {
        Internal, Absolute, External, None
    }

    class ImageDesc(val where: Where, val name: String) {
        val processedName = process()

        fun process(): String {
            var pName = name
            while (pName.contains("//")) {
                pName = pName.replace("//", "/")
            }
            return pName
        }

        override fun equals(other: Any?): Boolean {
            if (this === other) return true
            if (other?.javaClass != javaClass) return false

            other as ImageDesc

            if (where != other.where) return false
            if (processedName != other.processedName) return false

            return true
        }

        override fun hashCode(): Int {
            var result = where.hashCode()
            result = 31 * result + processedName.hashCode()
            return result
        }
    }
}