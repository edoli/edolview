package kr.edoli.edolview.asset

import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.image.ImageSpec

/**
 * Listening from network to get image
 */
class SocketAsset(imageName: String, private val bytes: ByteArray) : Asset() {

    override val name = "$imageName (Remote)"
    override val shouldAddToRecentAssets = true

    override fun retrieveImageSpec(): ImageSpec? {
        try {
            val mat = ImageConvert.bytesToMat(bytes)
            return ImageSpec(mat)
        } catch (e: Exception) {
            return null
        }
    }

    override fun next() = null

    override fun prev() = null

    override fun checkRefresh() = false
}