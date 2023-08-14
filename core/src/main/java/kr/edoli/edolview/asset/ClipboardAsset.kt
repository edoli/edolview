package kr.edoli.edolview.asset

import kr.edoli.edolview.image.ClipboardUtils
import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.image.ImageSpec
import java.awt.image.BufferedImage

/**
 * For paste from clipboard
 */
class ClipboardAsset : Asset() {

    companion object {
        private var counter = 0
    }

    override val name = "clipboard${counter}"
    override val shouldAddToRecentAssets = true

    init {
        counter += 1
    }

    override fun loadImageSpec(): ImageSpec? {
        val image = ClipboardUtils.getImage() as BufferedImage?
        if (image != null) {
            return ImageSpec(ImageConvert.bufferedToMat(image), 255.0, 8)
        }
        return null
    }

    override fun next(): Asset? = null

    override fun prev(): Asset? = null
    override fun checkRefresh() = false
}