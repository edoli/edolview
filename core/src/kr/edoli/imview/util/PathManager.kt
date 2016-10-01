package kr.edoli.imview.util

import com.badlogic.gdx.Gdx
import org.apache.commons.io.FileUtils
import org.apache.commons.io.FilenameUtils
import java.awt.datatransfer.MimeTypeParseException
import java.io.File
import java.io.IOException
import javax.activation.MimetypesFileTypeMap
import javax.imageio.ImageIO

/**
 * Created by daniel on 16. 10. 1.
 */
class PathManager {
    var currentPath: String? = null
    var currentFileName: String? = null

    fun setPath(path: String) {
        currentPath = FilenameUtils.getFullPath(path)
        currentFileName = FilenameUtils.getName(path)
    }

    fun next(): String? {
        if (currentPath == null || currentFileName == null) {
            return null
        }

        val files = Gdx.files.absolute(currentPath).list().map { it.name() }.sorted()
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

        val files = Gdx.files.absolute(currentPath).list().map { it.name() }.sorted()
        val currentIndex = files.indexOf(currentFileName)

        val index = currentIndex - 1
        currentFileName = if (index < files.size) files[files.size - 1] else files[index]
        val fullPath = currentPath + "/" + currentFileName

        if (!isImage(fullPath)) {
            return prev()
        }

        return fullPath
    }

    fun isImage(filePath: String): Boolean {
        try {
            ImageIO.read(File(filePath)) ?: return false
        } catch (ex: IOException) {
            return false
        }
        return true
    }
}