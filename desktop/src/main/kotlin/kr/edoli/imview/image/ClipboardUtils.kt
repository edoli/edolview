package kr.edoli.imview.image

import org.opencv.core.Mat
import java.awt.Image
import java.awt.Toolkit
import java.awt.datatransfer.DataFlavor
import java.awt.datatransfer.StringSelection
import java.awt.datatransfer.Transferable
import java.awt.datatransfer.UnsupportedFlavorException
import java.awt.image.BufferedImage
import javax.swing.ImageIcon
import javax.swing.JFrame
import javax.swing.JLabel


class ImageSelection(private val image: java.awt.Image) : Transferable {

    override fun getTransferDataFlavors(): Array<DataFlavor> {
        return arrayOf(DataFlavor.imageFlavor)
    }

    override fun isDataFlavorSupported(flavor: DataFlavor): Boolean {
        return DataFlavor.imageFlavor.equals(flavor)
    }

    override fun getTransferData(flavor: DataFlavor): java.awt.Image {
        if (!DataFlavor.imageFlavor.equals(flavor)) {
            throw UnsupportedFlavorException(flavor)
        }
        return image
    }
}

object ClipboardUtils {

    fun putImage(mat: Mat) {
        val buffered = ImageConvert.matToBuffered(mat)
        putImage(buffered)
    }

    fun putImage(buffered: BufferedImage) {
        val imgSel = ImageSelection(buffered)

        // Clipboard는 기본적으로 png, jpeg 모두 생성하려고 함.
        // channel 갯수가 4개 일때는 png는 생성되지만 jpeg에서는 에러가 발생
        // 해당 에러는 무시 가능
        Toolkit.getDefaultToolkit().systemClipboard.setContents(imgSel, null)
    }

    fun getImage(): Image? {
        val clipboard = Toolkit.getDefaultToolkit().systemClipboard
        val content = clipboard.getContents(null)
        if (!content.isDataFlavorSupported(DataFlavor.imageFlavor)) {
            return  null
        }
        return content.getTransferData(DataFlavor.imageFlavor) as Image?
    }

    fun showClipboardImage() {
        val image = getImage()
        if (image != null) {
            showImage(image)
        }
    }

    fun showImage(image: Image) {
        val frame = JFrame()
        frame.title = "Cropped"
        frame.add(JLabel().apply {
            icon = ImageIcon(image)
        })
        frame.pack()
        frame.isVisible = true
    }

    fun putString(text: String) {
        Toolkit.getDefaultToolkit().systemClipboard.setContents(StringSelection(text), null)
    }

    fun getString(): String {
        val clipboard = Toolkit.getDefaultToolkit().systemClipboard
        val content = clipboard.getContents(null)
        return content.getTransferData(DataFlavor.stringFlavor) as String? ?: ""
    }
}