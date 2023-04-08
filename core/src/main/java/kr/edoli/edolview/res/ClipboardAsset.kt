package kr.edoli.edolview.res

import kr.edoli.edolview.image.ClipboardUtils
import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.image.ImageSpec
import java.awt.image.BufferedImage

class ClipboardAsset : Asset() {

    override val name = "clipboard"
    override val shouldAddToRecentAssets = false

    override fun retrieveImageSpec(): ImageSpec? {
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