package kr.edoli.edolview.asset

import kr.edoli.edolview.image.ImageSpec
import org.opencv.core.Mat

/**
 * Listening from network to get image
 */
class SocketAsset(imageName: String, private val mat: Mat) : Asset() {

    override val name = "$imageName (Remote)"
    override val shouldAddToRecentAssets = true

    override fun loadImageSpec(): ImageSpec? {
        try {
            return ImageSpec(mat)
        } catch (e: Exception) {
            return null
        }
    }

    override fun next() = null

    override fun prev() = null

    override fun checkRefresh() = false
}