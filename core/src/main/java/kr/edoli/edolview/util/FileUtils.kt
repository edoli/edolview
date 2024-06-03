package kr.edoli.edolview.util

import org.opencv.core.MatOfByte
import java.io.File
import java.io.FileInputStream
import java.io.IOException
import java.nio.ByteOrder
import java.nio.channels.FileChannel
import java.nio.file.Files


object FileUtils {
    fun readFileToMatOfByte(file: File): MatOfByte {
        FileInputStream(file).use { fis ->
            val fileChannel: FileChannel = fis.channel
            val fileSize = fileChannel.size()

            if (fileSize > Int.MAX_VALUE) {
                throw IOException("File is too large to be processed")
            }

            val matOfByte = MatOfByte()
            matOfByte.alloc(fileSize.toInt())
            val buffer = NativeBuffer.wrapAddressByteBuffer(matOfByte.dataAddr(), fileSize.toInt())
            buffer.order(ByteOrder.nativeOrder())

            fileChannel.read(buffer)

            return matOfByte
        }
    }

    fun readFileToByteArray(file: File): ByteArray {
        return Files.readAllBytes(file.toPath())
    }
}