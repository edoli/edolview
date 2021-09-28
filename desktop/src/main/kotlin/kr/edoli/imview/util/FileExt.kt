package kr.edoli.imview.util

import org.apache.commons.io.FilenameUtils
import java.io.File
import kotlin.math.sign

/**
 * Created by daniel on 16. 9. 10.
 */

fun File.siblingFile(step: Int, fileList: List<String>? = null, filter: ((file: File) -> Boolean)? = null): File {
    if (step == 0) {
        return this
    }

    val file = this
    val absFile = file.absoluteFile

    if (absFile.parentFile == null) {
        return file
    }

    val fileNames = fileList ?: (absFile.parentFile.list() as Array<String>).sortedWith(FilenameComparator())
    val fileName = file.name
    val parentPath = absFile.parent + "/"

    var index = fileNames.indexOf(fileName)
    var path: String

    var validFiles = 0
    val stepDirection = step.sign
    repeat(fileNames.size) {
        index += stepDirection
        while (index >= fileNames.size) {
            index -= fileNames.size
        }

        while (index < 0) {
            index += fileNames.size
        }
        path = parentPath + fileNames[index]
        val nextFile = File(path)

        // Filtering
        filter?.let { filter ->
            if (!filter(nextFile) || !nextFile.exists()) {
                return@repeat
            }
        }

        validFiles += stepDirection

        if (validFiles != step) {
            return@repeat
        }

        return nextFile
    }

    return this
}

fun File.nextFile(interval: Int = 1, fileList: List<String>? = null, filter: ((file: File) -> Boolean)? = null): File {
    return siblingFile(interval, fileList, filter)
}

fun File.prevFile(interval: Int = 1, fileList: List<String>? = null, filter: ((file: File) -> Boolean)? = null): File {
    return siblingFile(-interval, fileList, filter)
}

fun File.getSiblingFiles(existFileList: List<String>? = null): List<String> {
    val absFile = this.absoluteFile
    val fileList = absFile.parentFile.list() as Array<String>
    if (existFileList == null || fileList.size != existFileList.size) {
        fileList.sortWith(FilenameComparator())
        return fileList.toList()
    }
    return existFileList
}
