package kr.edoli.imview.util

import org.apache.commons.io.FilenameUtils
import java.io.File

/**
 * Created by daniel on 16. 10. 1.
 */
class FileManager {
    val availableExts = arrayOf("png", "jpeg", "jpg", "jpe", "jp2", "bmp", "dib", "exr", "tif", "tiff", "hdr", "pic",
            "webp", "raw", "pfm", "pgm", "ppm", "pbm", "pxm", "pnm", "sr")

    private var currentFile: File? = null

    fun setFile(file: File) {
        currentFile = file
    }

    fun next(interval: Int): File? {
        currentFile?.let { file ->
            return file.nextFile(interval) { nextFile -> isImage(nextFile.name) }
        }
        return currentFile
    }

    fun prev(interval: Int): File? {
        currentFile?.let { file ->
            return file.prevFile(interval) { nextFile -> isImage(nextFile.name) }
        }
        return currentFile
    }

    fun isImage(filePath: String): Boolean {
        // Take too much time
        // try {
        //     ImageIO.read(File(filePath)) ?: return false
        // } catch (ex: IOException) {
        //     return false
        // }
        // return true

        val ext = FilenameUtils.getExtension(filePath).toLowerCase()
        return availableExts.contains(ext)
    }
}