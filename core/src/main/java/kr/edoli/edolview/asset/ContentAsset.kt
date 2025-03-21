package kr.edoli.edolview.asset

import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.image.ImageSpec
import kr.edoli.edolview.util.Platform

/**
 * For android content uri
 */
class ContentAsset(private val uri: String) : Asset() {

    override val name = uri
    override val shouldAddToRecentAssets = true

    override fun loadImageSpec(): ImageSpec? {
        val bytes = Platform.contentResolve(uri) ?: return null

        val mat = ImageConvert.decodeBytes(bytes)

        if (mat.width() <= 0 || mat.height() <= 0) {
            return null
        }

        val spec = ImageSpec(mat)
        spec.normalize()
        return spec
    }

    override fun next(): Asset? = null

    override fun prev(): Asset? = null
    override fun checkRefresh() = false
}