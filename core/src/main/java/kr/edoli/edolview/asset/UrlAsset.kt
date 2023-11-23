package kr.edoli.edolview.asset

import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.image.ImageSpec
import java.io.FileNotFoundException
import java.net.URL

/**
 * Retrieve image from url
 */
class UrlAsset(private val url: String) : Asset() {

    override val name = url
    override val shouldAddToRecentAssets = true

    override fun loadImageSpec(): ImageSpec? {
        val urlObj = URL(url)
        val bytes = try {
            urlObj.readBytes()
        } catch (e: FileNotFoundException) {
            // url file not found
            return null
        }
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