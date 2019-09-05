package kr.edoli.imview.util

import org.apache.commons.io.FilenameUtils
import java.io.File

/**
 * Created by daniel on 16. 10. 1.
 */
class FileManager {
    val availableExts = arrayOf("png", "jpeg", "jpg", "bmp", "exr", "pgm", "tif", "tiff", "hdr")

    private var currentFile: File? = null

    fun setFile(file: File) {
        currentFile = file
    }

    fun next(): File? {
        currentFile?.let { file ->
            return file.nextFile { nextFile -> isImage(nextFile.name) }
        }
        return currentFile
    }

    fun prev(): File? {
        currentFile?.let { file ->
            return file.prevFile { nextFile -> isImage(nextFile.name) }
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