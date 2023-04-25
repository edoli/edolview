package kr.edoli.edolview.util

import org.apache.commons.io.FilenameUtils
import java.io.File

/**
 * Created by daniel on 16. 10. 1.
 */
class FileManager {
    val availableExts = arrayOf("png", "jpeg", "jpg", "jpe", "jp2", "bmp", "dib", "exr", "tif", "tiff", "hdr", "pic",
            "webp", "raw", "pfm", "pgm", "ppm", "pbm", "pxm", "pnm", "sr", "flo")

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
            updateSiblingFiles(file)
            return file.nextFile(interval, siblingFiles) { nextFile -> isImageFile(nextFile) }
        }
        return currentFile
    }

    fun prev(interval: Int): File? {
        currentFile?.let { file ->
            updateSiblingFiles(file)
            return file.prevFile(interval, siblingFiles) { nextFile -> isImageFile(nextFile) }
        }
        return currentFile
    }

    fun updateSiblingFiles(file: File) {
        val files = siblingFiles

        // Optimize for folder with many files
        if (files != null && files.size > 200) {
            return
        }

        siblingFiles =  file.getSiblingFiles()
    }

    fun isInSameDirectory(file: File): Boolean {
        val cFile = currentFile ?: return false

        return file.parent == cFile.parent
    }

    fun isImageFile(file: File): Boolean {
        val name = file.name
        val ext = FilenameUtils.getExtension(name).lowercase()
        return file.exists() && availableExts.contains(ext)
    }
}