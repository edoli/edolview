package kr.edoli.imview.util

import org.apache.commons.io.FilenameUtils
import java.io.File

/**
 * Created by daniel on 16. 10. 1.
 */
class FileManager {
    val availableExts = arrayOf("png", "jpeg", "jpg", "jpe", "jp2", "bmp", "dib", "exr", "tif", "tiff", "hdr", "pic",
            "webp", "raw", "pfm", "pgm", "ppm", "pbm", "pxm", "pnm", "sr")

    var siblingFiles: List<String>? = null
    private var currentFile: File? = null

    fun setFile(file: File?) {
        currentFile = file
    }

    fun reset() {
        siblingFiles = null
    }

    fun next(interval: Int): File? {
        currentFile?.let { file ->
            if (siblingFiles == null) {
                siblingFiles = file.getSiblingFiles()
            }
            return file.nextFile(interval, siblingFiles) { nextFile -> isImage(nextFile.name) }
        }
        return currentFile
    }

    fun prev(interval: Int): File? {
        currentFile?.let { file ->
            if (siblingFiles == null) {
                siblingFiles = file.getSiblingFiles()
            }
            return file.prevFile(interval, siblingFiles) { nextFile -> isImage(nextFile.name) }
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