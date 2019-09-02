package kr.edoli.imview.ui

import com.sun.javafx.geom.BaseBounds
import com.sun.javafx.geom.transform.BaseTransform
import com.sun.javafx.scene.ImageViewHelper
import com.sun.javafx.sg.prism.NGImageView
import com.sun.javafx.sg.prism.NGNode
import com.sun.prism.Graphics
import com.sun.prism.Image
import com.sun.prism.Texture
import com.sun.prism.impl.BaseResourceFactory
import javafx.scene.Node
import javafx.scene.image.ImageView
import org.apache.commons.lang3.reflect.FieldUtils


class PixelatedImageView : ImageView() {

    init {
        try {
            initialize()
        } catch (e: IllegalAccessException) {
            e.printStackTrace()
        }
    }

    @Throws(IllegalAccessException::class)
    private fun initialize() {
        val nodeHelper = FieldUtils.readField(this, "nodeHelper", true)
        FieldUtils.writeField(nodeHelper, "imageViewAccessor", null, true)
        ImageViewHelper.setImageViewAccessor(object : ImageViewHelper.ImageViewAccessor {
            override fun doCreatePeer(node: Node): NGNode {
                return object : NGImageView() {
                    private var image: Image? = null

                    override fun setImage(img: Any) {
                        super.setImage(img)
                        image = img as Image
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

            override fun doUpdatePeer(node: Node) {
                val method = ImageView::class.java.getDeclaredMethod("doUpdatePeer")
                method.isAccessible = true
                method.invoke(node)
            }

            override fun doComputeGeomBounds(node: Node,
                                             bounds: BaseBounds, tx: BaseTransform): BaseBounds {
                val method = ImageView::class.java.getDeclaredMethod("doComputeGeomBounds", BaseBounds::class.java, BaseTransform::class.java)
                method.isAccessible = true
                return method.invoke(node, bounds, tx) as BaseBounds
            }

            override fun doComputeContains(node: Node, localX: Double, localY: Double): Boolean {
                val method = ImageView::class.java.getDeclaredMethod("doComputeContains", Double::class.java, Double::class.java)
                method.isAccessible = true
                return method.invoke(node, localX, localY) as Boolean
            }
        })
    }
}