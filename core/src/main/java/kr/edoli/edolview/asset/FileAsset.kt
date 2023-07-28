package kr.edoli.edolview.asset

import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.ImageSpec
import kr.edoli.edolview.store.ImageStore
import kr.edoli.edolview.util.FileManager
import java.io.File

/**
 * File content
 */
class FileAsset(val path: String) : Asset() {
    companion object {
        val fileManager = FileManager()
    }

    var currentFile: File? = null
        set(file) {
            // Update last modified when file updated
            currentFileLastModified = file?.lastModified() ?: 0

            // heck is file in same directory
            if (file != null && !fileManager.isInSameDirectory(file)) {
                fileManager.reset()
            }

            field = file
        }
    var currentFileDirectory: String? = null
    var currentFileLastModified = 0L

    override val name: String
        get() = path

    override val shouldAddToRecentAssets = true

    override val workingDirectory: String
        get() = currentFileDirectory ?: ""

    override fun load(): ImageSpec? {

        val file = File(path)

        if (!fileManager.isImageFile(file)) {
            return null
        }

        if (file.isDirectory) {
            fileManager.setFile(file)

        } else if (file.isFile) {
            currentFile = file
            currentFileDirectory = file.absoluteFile.parent
            fileManager.setFile(file)
            val spec = ImageStore.get(file)
            val mat = spec.mat
            if (!mat.empty()) {
                return spec
            }
        }
//        else {
//            // Not from file. Clear file manager
//            mainFile.update(null)
//            mainFileName.update(path)
//            mainFileDirectory.update("")
//            ImContext.fileManager.setFile(null)
//        }

        return null
    }

    override fun next(): Asset? {
        val nextFile = fileManager.next(ImContext.frameInterval.get())
        if (nextFile != null) {
            return FileAsset(nextFile.path)
        }
        return null
    }

    override fun prev(): Asset? {
        val prevFile = fileManager.prev(ImContext.frameInterval.get())
        if (prevFile != null) {
            return FileAsset(prevFile.path)
        }
        return null
    }

    override fun checkRefresh(): Boolean {
        return ((currentFile?.lastModified() ?: 0L) > currentFileLastModified)
    }
}