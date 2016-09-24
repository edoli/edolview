package kr.edoli.imview.ui.view

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.FileDropMessage
import kr.edoli.imview.util.Windows
import java.util.*

/**
 * Created by daniel on 16. 9. 24.
 */
class ImageListViewer : Table() {

    private val imageList = ArrayList<Pixmap>()
    private val imageSummaryList = ArrayList<ImageSummary>()

    private var isRefresh = false

    private var size = 128f

    init {
        align(Align.topLeft)

        Bus.subscribe(FileDropMessage::class.java) {
            if (windowName == Windows.ImageList) {
                for (file in files) {
                    addImagePath(file)
                }
            }
        }
    }

    override fun act(delta: Float) {
        if (isRefresh) {
            refresh()
            isRefresh = false
        }

        super.act(delta)
    }

    fun refresh() {
        while (imageSummaryList.size < imageList.size) {
            imageSummaryList.add(ImageSummary())
        }

        clearChildren()

        for ((index, pixmap) in imageList.withIndex()) {
            val texture = Texture(pixmap)
            texture.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)
            imageSummaryList[index].region = TextureRegion(texture)
        }

        val numCol = (width / size).toInt()

        for ((index, imageSummary) in imageSummaryList.withIndex()) {
            add(imageSummary).size(size)

            if (index % numCol == numCol - 1) {
                row()
            }
        }
    }


    fun addImagePath(path: String) {
        val pixmap = Pixmap(Gdx.files.absolute(path))

        addImage(pixmap)
    }

    fun addImage(pixmap: Pixmap) {
        imageList.add(pixmap)

        isRefresh = true
    }

    fun increaseSize() {
        size * 1.2f
    }

    fun decreaseSize() {
        size / 1.2f
    }

    class ImageSummary : Actor() {

        var region: TextureRegion? = null

        override fun draw(batch: Batch, parentAlpha: Float) {
            if (region != null) {
                val regionWidth = (region as TextureRegion).regionWidth
                val regionHeight = (region as TextureRegion).regionHeight

                val aspectRatio = regionWidth.toFloat() / regionHeight.toFloat()

                if (aspectRatio > 1) {
                    val pad = (height - (width / aspectRatio)) / 2
                    batch.draw(region, x, y + pad, width, height - pad * 2)
                } else {
                    val pad = (width - (height * aspectRatio)) / 2
                    batch.draw(region, x + pad, y, width - pad * 2, height)
                }
            }
        }
    }
}