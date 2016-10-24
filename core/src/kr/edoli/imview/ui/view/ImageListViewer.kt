package kr.edoli.imview.ui.view

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.math.Rectangle
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.Group
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.FileDropMessage
import kr.edoli.imview.ui.UI
import kr.edoli.imview.util.Windows
import kr.edoli.imview.util.getChannels
import org.apache.commons.io.FilenameUtils
import java.util.*

/**
 * Created by daniel on 16. 9. 24.
 */
class ImageListViewer : Table() {

    private val imageSummaryList = ArrayList<ImageSummary>()
    private val imageSummaryViewList = ArrayList<ImageSummaryView>()

    private var isRefresh = false

    private var cellWidth = 128f
    private var cellHeight = 128f

    init {
        align(Align.topLeft)

        Bus.subscribe(FileDropMessage::class.java) {
            if (windowName == Windows.ImageList) {
                for (file in files) {
                    addImagePath(file)
                }
            }
        }

        Context.mainImage.subscribe {
            if (it != null) {
                for (imageSummary in imageSummaryList) {
                    if (imageSummary.pixmap != null) {
                        imageSummary.psnr = psnr(it, imageSummary.pixmap!!)
                    }
                }
            }
            update()
        }

        Context.selectBox.subscribe {
            val mainImage = Context.mainImage.get()
            if (mainImage != null) {
                for (imageSummary in imageSummaryList) {
                    if (imageSummary.pixmap != null) {
                        imageSummary.psnr = psnr(mainImage, imageSummary.pixmap!!)
                    }
                }
            }
            update()
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
        while (imageSummaryViewList.size < imageSummaryList.size) {
            imageSummaryViewList.add(ImageSummaryView())
        }

        clearChildren()

        for ((index, imageSummary) in imageSummaryList.withIndex()) {
            val pixmap = imageSummary.pixmap
            val texture = Texture(pixmap)
            texture.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)
            imageSummaryViewList[index].imageSummary = imageSummary
        }

        val numCol = (width / cellWidth).toInt()

        for ((index, imageSummary) in imageSummaryViewList.withIndex()) {
            add(imageSummary).size(cellWidth, cellHeight)

            if (index % numCol == numCol - 1) {
                row()
            }
        }
    }

    fun update() {
        for (imageSummaryView in imageSummaryViewList) {
            imageSummaryView.update()
        }
    }


    fun addImagePath(path: String) {
        val pixmap = Pixmap(Gdx.files.absolute(path))

        addImage(pixmap, FilenameUtils.getBaseName(path))
    }

    fun addImage(pixmap: Pixmap, name: String) {
        val imageSummary = ImageSummary()

        var imageRegion =  if (pixmap.format == Pixmap.Format.Alpha) TextureRegion(Texture(pixmap, Pixmap.Format.LuminanceAlpha, false))
        else TextureRegion(Texture(pixmap))

        imageSummary.pixmap = pixmap
        imageSummary.region = TextureRegion(imageRegion)
        imageSummary.title = name

        val mainPixmap = Context.mainImage.get()
        if (mainPixmap != null) {
            imageSummary.psnr = psnr(mainPixmap, pixmap)
        }

        imageSummaryList.add(imageSummary)

        isRefresh = true
    }

    fun psnr(pixmapA: Pixmap, pixmapB: Pixmap, rect: Rectangle): Double {
        val pixelsA = pixmapA.pixels
        val pixelsB = pixmapB.pixels

        if (pixmapA.width != pixmapB.width || pixmapA.height != pixmapB.height || pixmapA.format != pixmapB.format) {
            return -1.0;
        }

        val x = rect.x.toInt()
        val y = rect.y.toInt()
        val width = rect.width.toInt()
        val height = rect.height.toInt()

        val channels = pixmapA.getChannels()

        val size = width * height * channels

        var mse = 0.0

        for (tx in x..x+width-1) {
            for (ty in y..y+height-1) {
                val ind = (tx + ty * width) * channels

                for (c in 0..channels-1) {
                    val indc = ind + c
                    val value = (pixelsA[indc] - pixelsB[indc]).toByte()

                    mse += value * value
                }
            }
        }

        mse /= size

        return 20 * Math.log10(255.0) - 10 * Math.log10(mse)
    }

    fun psnr(pixmapA: Pixmap, pixmapB: Pixmap): Double {
        val selectBox = Context.selectBox.get()
        if (selectBox.width != 0f && selectBox.height != 0f) {
            return psnr(pixmapA, pixmapB, selectBox)
        }
        return psnr(pixmapA, pixmapB, Rectangle(0f, 0f, pixmapA.width.toFloat(), pixmapA.height.toFloat()))
    }

    class ImageSummary {
        var pixmap: Pixmap? = null
        var region: TextureRegion? = null
        var title = ""
        var psnr: Double = 0.0
    }

    class ImageSummaryView : Table() {
        val psnrLabel = UI.label("")
        val titleLabel = UI.label("")
        val imageAspectView = ImageAspectView()

        var imageSummary: ImageSummary? = null
            set(value) {
                field = value
                update()
            }

        init {
            psnrLabel.setFontScale(0.75f)
            titleLabel.setFontScale(0.75f)

            psnrLabel.setAlignment(Align.center)
            titleLabel.setAlignment(Align.center)

            clip = true

            add(titleLabel).height(24f).expandX().fillX()
            row()
            add(imageAspectView).expand().fill()
            row()
            add(psnrLabel).height(24f).expandX().fillX()
            update()
        }

        fun update() {
            if (imageSummary != null) {
                titleLabel.setText(imageSummary!!.title)

                val psnr = imageSummary!!.psnr
                if (psnr > 0) {
                    psnrLabel.setText("PSNR: ${String.format("%.2f", psnr)}")
                } else {
                    psnrLabel.setText("image size not match")
                }
                imageAspectView.region = imageSummary!!.region
            }
        }
    }

    class ImageAspectView : Actor() {
        var region: TextureRegion? = null

        override fun draw(batch: Batch, parentAlpha: Float) {
            if (region != null) {
                val regionWidth = region!!.regionWidth
                val regionHeight = region!!.regionHeight

                val aspectRatio = regionWidth.toFloat() / regionHeight.toFloat()

                if (aspectRatio > 1) {
                    val pad = (height - (width / aspectRatio)) / 2
                    batch.draw(region, x, y + pad, width, height - pad * 2)
                } else {
                    val pad = (width - (height * aspectRatio)) / 2
                    batch.draw(region, x + pad, y, width - pad * 2, height)
                }

            }

            super.draw(batch, parentAlpha)
        }
    }
}