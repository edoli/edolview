package kr.edoli.edolview.res

import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.image.ImageSpec
import java.io.FileNotFoundException
import java.net.URL

class UrlAsset(private val url: String) : Asset() {

    override val name = url
    override val shouldAddToRecentAssets = true

    override fun retrieveImageSpec(): ImageSpec? {
        val urlObj = URL(url)
        val bytes = try {
            urlObj.readBytes()
        } catch (e: FileNotFoundException) {
            // url file not found
            return null
        }
        val mat = ImageConvert.bytesToMat(bytes)

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