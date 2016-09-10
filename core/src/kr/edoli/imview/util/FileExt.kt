package kr.edoli.imview.util

import org.apache.commons.io.FilenameUtils
import java.io.File
import java.util.*

/**
 * Created by daniel on 16. 9. 10.
 */

fun File.siblingFile(modifier: Int) : File {

    val file = this

    if (file.parentFile == null) {
        return file
    }

    val fileNames = file.parentFile.list()
    val fileName = file.name
    fileNames.sort()

    var index = fileNames.indexOf(fileName)
    var path: String

    do {
        index += modifier

        if (index == fileNames.size) {
            index = 0
        }
        if (index == -1) {
            index = fileNames.size - 1
        }

        path = file.parent + "/" + fileNames[index]
    } while (!File(path).isImage())

    return File(path)
}

fun File.nextFile() : File {
    return siblingFile(1)
}

fun File.prevFile() : File {
    return siblingFile(-1)
}

fun File.isImage() : Boolean {
    val ext = FilenameUtils.getExtension(path)
    val exts = arrayOf("png", "PNG", "jpg", "JPG")

    return exts.contains(ext)
}