package kr.edoli.imview.ui

import javafx.scene.layout.VBox
import kr.edoli.imview.ImContext
import kr.edoli.imview.util.ImageInfo
import org.apache.commons.io.FileUtils
import org.opencv.core.Mat
import tornadofx.addClass
import tornadofx.clear
import tornadofx.label
import tornadofx.region
import java.io.File
import java.nio.file.Files
import java.nio.file.Paths
import java.nio.file.attribute.BasicFileAttributes
import java.text.DecimalFormat
import java.text.SimpleDateFormat

class FileInfoPane : VBox() {

    private val dateFormatter = SimpleDateFormat("yyyy.MM.dd hh:mm:ss")
    private val numberFomatter = DecimalFormat("#,###.##")

    init {
        addClass(Style.overlayPane)

        ImContext.mainImage.subscribe {
            val image = it
            if (image != null) {
                clear()
                refresh(image)
            }
        }
    }

    private fun refresh(image: Mat) {

        if (ImContext.mainPath.get() == "") {
            return
        }
        val path = ImContext.mainPath.get()


        val info = ImageInfo()
        info.setInput(FileUtils.openInputStream(File(path)))
        info.check()

        addItem("dimension",  "${image.cols()} × ${image.rows()}")

        addItem("image format", info.formatName)

        addItem("bits per pixel", (ImContext.imageSpec.get()?.bitsPerPixel ?: -1).toString() + " bits")

        val attr = Files.readAttributes(Paths.get(path), BasicFileAttributes::class.java)

        addItem("size", numberFomatter.format(attr.size().toDouble() / 1024 / 1024) + " mb", "(${numberFomatter.format(attr.size())} bytes)")

        addItem("created date", dateFormatter.format(attr.creationTime().toMillis()))

        addItem("last modified date", dateFormatter.format(attr.lastModifiedTime().toMillis()))

        addItem("last accessed date", dateFormatter.format(attr.lastAccessTime().toMillis()))
    }

    private fun addItem(title: String, content: String, subContent: String = "") {
        label(title) {
            addClass(Style.boldLabel)
        }
        label(content)

        if (subContent != "") {
            label(subContent)
            region { prefHeight = 2.0 }
        } else {
            region { prefHeight = 8.0 }
        }
    }
}