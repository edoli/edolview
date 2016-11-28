package kr.edoli.imview.store

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Pixmap
import com.google.common.cache.CacheBuilder
import com.google.common.cache.CacheLoader

/**
 * Created by daniel on 16. 11. 27.
 */
object ImageStore {

    private val MAX_MEMORY = 1024 * 1024 * 1024L


    private val imageStoreMap = CacheBuilder.newBuilder()
            .maximumWeight(MAX_MEMORY)
            .weigher { k1: ImageDesc, v1: Pixmap? -> v1?.pixels?.limit() ?: 0 }
            .removalListener{ it.value?.dispose() }
            .build(object : CacheLoader<ImageDesc, Pixmap?>() {
                override fun load(key: ImageDesc): Pixmap? {
                    when (key.where) {
                        Where.Absolute -> return Pixmap(Gdx.files.absolute(key.name))
                        Where.Internal -> return Pixmap(Gdx.files.internal(key.name))
                        Where.External -> return Pixmap(Gdx.files.external(key.name))
                    }
                    return null
                }
            })

    fun get(where: Where, name: String): Pixmap? {
        var processedName = name
        while (processedName.contains("//")) {
            processedName = processedName.replace("//", "/")
        }
        return imageStoreMap[ImageDesc(where, name)]
    }

    fun put(name: String, pixmap: Pixmap) {
        val desc = ImageDesc(Where.None, name)
        imageStoreMap[desc]?.dispose()
        imageStoreMap.put(desc, pixmap)
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