package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.utils.Align
import kr.edoli.edoliui.widget.drawable.ColorDrawable
import kr.edoli.imview.Context
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.UI
import kr.edoli.imview.util.ImageInfo
import kr.edoli.imview.util.getChannels
import org.apache.commons.io.FileUtils
import java.io.File
import java.nio.file.Files
import java.nio.file.Paths
import java.nio.file.attribute.BasicFileAttributes
import java.text.DecimalFormat
import java.text.SimpleDateFormat


/**
 * Created by daniel on 16. 11. 27.
 */
class InfoView : Table() {

    val titleSize = 20
    val contentSize = 24
    val subContentSize = 16
    val titleColor = Color.valueOf("#cccccc")!!

    val dateFormatter = SimpleDateFormat("yyyy.MM.dd hh:mm:ss")
    val numberFomatter = DecimalFormat("#,###.##")

    var needRefresh = false

    init {
        clip = true
        background = ColorDrawable(Colors.overlayBackground)

        pad(16f, 16f, 0f, 16f)
        align(Align.top)

        Context.mainImage.subscribe {
            needRefresh = true
        }
        Context.mainPath.subscribe {
            needRefresh = true
        }
    }

    fun refresh() {
        clearChildren()

        val image = Context.mainImage.get() ?: return

        addItem("dimension",  "${image.cols()} × ${image.rows()}")

        addItem("number of channels", image.channels().toString())


        if (Context.mainPath.get() == "") {
            return
        }
        val path = Context.mainPath.get()


        val info = ImageInfo()
        info.setInput(FileUtils.openInputStream(File(path)))
        info.check()

        addItem("image format", info.formatName)

        addItem("bits per pixel", info.bitsPerPixel.toString() + " bits")

        val attr = Files.readAttributes(Paths.get(path), BasicFileAttributes::class.java)

        addItem("size", numberFomatter.format(attr.size().toDouble() / 1024 / 1024) + " mb", "(" + numberFomatter.format(attr.size()) + " bytes)")

        addItem("created date", dateFormatter.format(attr.creationTime().toMillis()))

        addItem("last modified date", dateFormatter.format(attr.lastModifiedTime().toMillis()))

        addItem("last accessed date", dateFormatter.format(attr.lastAccessTime().toMillis()))

    }

    fun addItem(title: String, content: String, subContent: String = "") {
        add(UI.label(title, size=titleSize).apply { color = titleColor }).align(Align.left).expandX().padBottom(8f).row()
        add(UI.label(content, size= contentSize)).align(Align.left).expandX().row()
        if (subContent != "") {
            add(UI.label(subContent, size= subContentSize)).align(Align.left).expandX().padTop(4f).row()
        }
        add().height(16f).row()
    }

    override fun act(delta: Float) {
        if (needRefresh) {
            refresh()
            needRefresh = false
        }
        super.act(delta)
    }

    override fun getMinHeight(): Float {
        return 0f
    }
}