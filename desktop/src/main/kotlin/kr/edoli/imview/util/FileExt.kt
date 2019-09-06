package kr.edoli.imview.util

import org.apache.commons.io.FilenameUtils
import java.io.File

/**
 * Created by daniel on 16. 9. 10.
 */

fun File.siblingFile(step: Int, filter: ((file: File) -> Boolean)? = null): File {

    val file = this
    val absFile = file.absoluteFile

    if (absFile.parentFile == null) {
        return file
    }

    val fileNames = (absFile.parentFile.list() as Array<String>).sortedWith(FilenameComparator())
    val fileName = file.name
    val parentPath = absFile.parent + "/"

    var index = fileNames.indexOf(fileName)
    var path: String

    repeat(fileNames.size) {
        index += step
        if (index == fileNames.size) {
            index = 0
        }

        if (index == -1) {
            index = fileNames.size - 1
        }
        path = parentPath + fileNames[index]
        val nextFile = File(path)

        // Filtering
        filter?.let { filter ->
            if (!filter(nextFile) || !nextFile.exists()) {
                return@repeat
            }
        }

        return nextFile
    }

    return this
}

fun File.nextFile(filter: ((file: File) -> Boolean)? = null): File {
    return siblingFile(1, filter)
}

fun File.prevFile(filter: ((file: File) -> Boolean)? = null): File {
    return siblingFile(-1, filter)
}