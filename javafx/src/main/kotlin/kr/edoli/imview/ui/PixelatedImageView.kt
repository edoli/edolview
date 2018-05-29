package kr.edoli.imview.ui

import com.sun.javafx.sg.prism.NGImageView
import com.sun.javafx.sg.prism.NGNode
import com.sun.prism.Graphics
import com.sun.prism.Image
import com.sun.prism.Texture
import com.sun.prism.impl.BaseResourceFactory
import javafx.scene.image.ImageView

class PixelatedImageView : ImageView() {
    override fun impl_createPeer(): NGNode {
        return object : NGImageView() {
            private var image: Image? = null

            override fun setImage(img: Any?) {
                super.setImage(img)
                if (img != null) {
                    image = img as Image
                }
            }

            override fun renderContent(g: Graphics) {
                val factory = g.resourceFactory as BaseResourceFactory
                val tex = factory.getCachedTexture(image, Texture.WrapMode.CLAMP_TO_EDGE)
                tex.linearFiltering = false
                tex.unlock()
                super.renderContent(g)
            }
        }
    }
}