package kr.edoli.edolview.util

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
            siblingFiles = file.getSiblingFiles(siblingFiles)
            return file.nextFile(interval, siblingFiles) { nextFile -> isImageFile(nextFile) }
        }
        return currentFile
    }

    fun prev(interval: Int): File? {
        currentFile?.let { file ->
            siblingFiles = file.getSiblingFiles(siblingFiles)
            return file.prevFile(interval, siblingFiles) { nextFile -> isImageFile(nextFile) }
        }
        return currentFile
    }

    fun isInSameDirectory(file: File): Boolean {
        val cFile = currentFile ?: return false

        return file.parent == cFile.parent
    }

    fun isImageFile(file: File): Boolean {
        // Take too much time
        // try {
        //     ImageIO.read(File(filePath)) ?: return false
        // } catch (ex: IOException) {
        //     return false
        // }
        // return true

        val name = file.name
        val ext = FilenameUtils.getExtension(name).lowercase()
        return file.exists() && availableExts.contains(ext)
    }
}