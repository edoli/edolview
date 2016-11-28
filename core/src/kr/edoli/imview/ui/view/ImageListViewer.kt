package kr.edoli.imview.ui.view

import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.TextureRegion
import com.badlogic.gdx.graphics.glutils.ShapeRenderer
import com.badlogic.gdx.math.Rectangle
import com.badlogic.gdx.scenes.scene2d.*
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.Context
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.FileDropMessage
import kr.edoli.imview.res.FontAwesomes
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.ui.UI
import kr.edoli.imview.ui.onClick
import kr.edoli.imview.util.Clipboard
import kr.edoli.imview.image.ImageProc
import kr.edoli.imview.util.Windows
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
    private var cellHeight = 176f

    init {
        align(Align.topLeft)
        touchable = Touchable.enabled

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
                        imageSummary.metric = Context.comparisonMetric.get().compute(it, imageSummary.pixmap!!, Context.selectBox.get())
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
                        imageSummary.metric = Context.comparisonMetric.get().compute(mainImage, imageSummary.pixmap!!, Context.selectBox.get())
                    }
                }
            }
            update()
        }

        addListener(object : ClickListener(Input.Buttons.LEFT) {
            override fun clicked(event: InputEvent, x: Float, y: Float) {
                if (event.target == this@ImageListViewer) {
                    Context.selectedImage.update(null)
                    update()
                }
                super.clicked(event, x, y)
            }
        })
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
            val summaryView = imageSummaryViewList[index]
            texture.setFilter(Texture.TextureFilter.Linear, Texture.TextureFilter.Linear)
            summaryView.imageSummary = imageSummary
            summaryView.isUsed = true


            summaryView.clearListeners()
            summaryView.onClick {
            }
            summaryView.addListener(object : ClickListener() {
                override fun touchDown(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                    return super.touchDown(event, x, y, pointer, button)
                }

                override fun touchUp(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int) {
                    Context.selectedImage.update(imageSummary!!.pixmap)
                    update()
                    super.touchUp(event, x, y, pointer, button)
                }
                override fun clicked(event: InputEvent?, x: Float, y: Float) {
                    super.clicked(event, x, y)
                }
            })


            val s = summaryView.deleteButton.listeners.size
            if (s > 2) {
                summaryView.deleteButton.listeners.removeRange(2, s - 1)
            }
            summaryView.deleteButton.onClick {
                imageSummaryList.remove(imageSummary)
                refresh()
            }
        }

        for (i in imageSummaryList.size..imageSummaryViewList.size-1) {
            imageSummaryViewList[i].isUsed = false
        }

        val numCol = (width / cellWidth).toInt()

        var index = 0
        for (imageSummary in imageSummaryViewList) {
            if (!imageSummary.isUsed) {
                continue
            }

            add(imageSummary).size(cellWidth, cellHeight)

            if (index % numCol == numCol - 1) {
                row()
            }
            index += 1
        }
    }

    override fun sizeChanged() {
        refresh()
        super.sizeChanged()
    }

    fun update() {
        for (imageSummaryView in imageSummaryViewList) {
            imageSummaryView.update()
        }
    }


    fun addImagePath(path: String) {
        val pixmap = ImageStore.get(ImageStore.Where.Absolute, path)

        if (pixmap != null) {
            addImage(pixmap, FilenameUtils.getBaseName(path))
        }
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
            imageSummary.metric = Context.comparisonMetric.get().compute(mainPixmap, pixmap, Context.selectBox.get())
        }

        imageSummaryList.add(imageSummary)

        isRefresh = true
    }

    class ImageSummary {
        var pixmap: Pixmap? = null
        var region: TextureRegion? = null
        var title = ""
        var metric: Double = 0.0
    }

    class ImageSummaryView : Table() {
        companion object {
            val shapeRenderer = ShapeRenderer()
            init {
                shapeRenderer.color = Color.WHITE
            }
        }

        val psnrLabel = UI.label("")
        val titleLabel = UI.label("")
        val deleteButton = UI.iconButton(FontAwesomes.FaRemove)
        val imageAspectView = ImageAspectView()

        var isUsed = false
        var isSelected = false
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

            val titleBar = Table()

            titleBar.add(titleLabel).height(24f).expandX().fillX()
            titleBar.add(deleteButton).size(24f)

            val psnrCopyButton = UI.iconButton(FontAwesomes.FaCopy).onClick {
                if (imageSummary != null) {
                    Clipboard.copy(String.format("%.2f", (imageSummary as ImageSummary).metric))
                }
            }

            add(titleBar).expandX().fillX().height(24f)
            row()
            add(imageAspectView).expand().fill()
            row()
            add(Table().apply {
                add(psnrCopyButton).size(24f).padRight(8f)
                add(psnrLabel).height(24f).width(96f)
            }).expandX().fillX()
            update()
        }

        fun dispose() {
            imageSummary?.region?.texture?.dispose()
        }

        fun update() {
            if (imageSummary != null) {
                titleLabel.setText(imageSummary!!.title)

                val psnr = imageSummary!!.metric
                if (psnr > 0) {
                    psnrLabel.setText("PSNR: ${String.format("%.2f", psnr)}")
                } else {
                    psnrLabel.setText("image size not match")
                }

                imageAspectView.region = imageSummary!!.region

                if (Context.selectedImage.get() == imageSummary!!.pixmap) {
                    isSelected = true
                } else {
                    isSelected = false
                }
            } else {
                titleLabel.setText("")
                imageAspectView.region = null
                isSelected = false
            }
        }

        override fun draw(batch: Batch, parentAlpha: Float) {
            super.draw(batch, parentAlpha)

            if (isSelected) {
                batch.end()
                shapeRenderer.begin(ShapeRenderer.ShapeType.Line)
                shapeRenderer.rect(x + 1, y + 1, width - 2, height - 2)
                shapeRenderer.end()
                batch.begin()
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