package kr.edoli.imview.util

import org.apache.commons.io.FilenameUtils
import java.io.File

/**
 * Created by daniel on 16. 10. 1.
 */
class PathManager {
    val availableExts = arrayOf("png", "jpeg", "jpg", "bmp", "exr", "pgm", "tif", "tiff", "hdr")

    private var currentPath: String? = null
    private var currentFileName: String? = null


    fun setPath(path: String) {
        val absPath = File(path).absolutePath
        currentPath = FilenameUtils.getFullPath(absPath)
        currentFileName = FilenameUtils.getName(absPath)
    }

    fun next(): String? {
        if (currentPath == null || currentFileName == null) {
            return null
        }

        val files = File(currentPath).list().map { it }.sortedWith(FilenameComparator())
        val currentIndex = files.indexOf(currentFileName)

        val index = currentIndex + 1
        currentFileName = if (index >= files.size) files[0] else files[index]
        val fullPath = currentPath + "/" + currentFileName

        if (!isImage(fullPath)) {
            return next()
        }

        return fullPath
    }

    fun prev(): String? {
        if (currentPath == null || currentFileName == null) {
            return null
        }

        val files = File(currentPath).list().map { it }.sortedWith(FilenameComparator())
        val currentIndex = files.indexOf(currentFileName)

        val index = currentIndex - 1
        currentFileName = if (index < 0) files[files.size - 1] else files[index]
        val fullPath = currentPath + "/" + currentFileName

        if (!isImage(fullPath)) {
            return prev()
        }

        return fullPath
    }

    fun isImage(filePath: String): Boolean {
        /*
        try {
            ImageIO.read(File(filePath)) ?: return false
        } catch (ex: IOException) {
            return false
        }
        return true*/
        val ext = FilenameUtils.getExtension(filePath).toLowerCase()
        return availableExts.contains(ext)
    }
}