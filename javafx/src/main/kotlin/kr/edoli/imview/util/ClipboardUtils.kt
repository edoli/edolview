package kr.edoli.imview.util

import javafx.scene.Scene
import javafx.scene.control.Label
import javafx.scene.image.Image
import javafx.scene.image.ImageView
import javafx.scene.input.Clipboard
import javafx.scene.input.ClipboardContent
import javafx.stage.Stage
import kr.edoli.imview.image.ImageConvert
import org.opencv.core.Mat

object ClipboardUtils {

    fun putImage(mat: Mat) {
        putImage(ImageConvert.matToImage(mat))
    }

    fun putImage(image: Image) {
        val clipboard = Clipboard.getSystemClipboard()
        val content = ClipboardContent()
        content.putImage(image)
        clipboard.setContent(content)
    }

    fun getImage(): Image? {
        val clipboard = Clipboard.getSystemClipboard()
        return clipboard.image
    }

    fun showClipboardImage() {
        val image = getImage()
        if (image != null) {
            showImage(image)
        }
    }

    fun showImage(image: Image) {
        val imageLabel = Label()
        imageLabel.graphic = ImageView(image)

        val scene = Scene(imageLabel)
        val stage = Stage()
        stage.scene = scene
        stage.show()
    }

    fun putString(text: String) {
        val clipboard = Clipboard.getSystemClipboard()
        val content = ClipboardContent()
        content.putString(text)
        clipboard.setContent(content)
    }

    fun getString(): String {
        val clipboard = Clipboard.getSystemClipboard()
        return clipboard.string
    }
}