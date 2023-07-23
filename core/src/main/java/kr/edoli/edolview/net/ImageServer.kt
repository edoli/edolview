package kr.edoli.edolview.net

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Net
import com.badlogic.gdx.net.Socket
import com.badlogic.gdx.utils.GdxRuntimeException
import com.badlogic.gdx.utils.Json
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.asset.SocketAsset
import kr.edoli.edolview.image.ImageConvert
import kr.edoli.edolview.util.CustomByteArrayOutputStream
import org.opencv.core.CvType
import org.opencv.core.Mat
import java.io.BufferedInputStream
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.charset.StandardCharsets
import java.util.zip.Inflater
import kotlin.concurrent.thread

class ImageServer(host: String, port: Int) {

    companion object {
        fun create() {
            thread {
                createServer(21734)
            }
        }

        private fun createServer(port: Int) {
            try {
                val server = ImageServer("127.0.0.1", port)
                ImContext.imageServerAddress.update("127.0.0.1:${port}")
                server.start()
            } catch (e: GdxRuntimeException) {
                createServer(port + 1)
            }
        }
    }

    private val server = Gdx.net.newServerSocket(Net.Protocol.TCP, host, port, null)

    fun start() {
        while (true) {
            val socket = server.accept(null)

            if (socket != null && ImContext.isServerActive.get()) {
                thread { ImageHandler(socket).start() }
            }
        }
    }
}

class Extra {
    var shape: IntArray = intArrayOf()
    var dtype: String = ""
    var compression: String = ""
    var nbytes: Int = 0
}

class ImageHandler(socket: Socket) {

    private val reader = BufferedInputStream(socket.inputStream)

    fun start() {
        val nameSize = readInt()
        val extraSize = readInt()
        val bufferSize = readInt()

        val name = readString(nameSize)
        val extraStr = readString(extraSize)

        val extra = Json().fromJson(Extra::class.java, extraStr) ?: return
        val shape = extra.shape
        val dtype = extra.dtype
        val numChannel = shape[2]

        // Read image buffer
        val bufferBytes = ByteArray(bufferSize)

        var remain = bufferSize
        var currentPosition = 0

        while (remain > 0) {
            val recvSize = reader.read(bufferBytes, currentPosition, remain)

            remain -= recvSize
            currentPosition += recvSize
        }

        val mat = when (extra.compression) {
            "zlib" -> {
                val inflater = Inflater()
                val outputStream = CustomByteArrayOutputStream(extra.nbytes)

                val buffer = ByteArray(4096)

                inflater.setInput(bufferBytes)

                while (!inflater.finished()) {
                    val count = inflater.inflate(buffer)
                    outputStream.write(buffer, 0, count)

                }

                outputStream.close()

                val byteBuffer = ByteBuffer.allocateDirect(extra.nbytes)
                byteBuffer.order(ByteOrder.nativeOrder())
                byteBuffer.put(outputStream.getBuf())

                when (dtype) {
                    "float64" -> Mat(shape[0], shape[1], CvType.CV_64FC(numChannel), byteBuffer)
                    "float32" -> Mat(shape[0], shape[1], CvType.CV_32FC(numChannel), byteBuffer)
                    "float16" -> Mat(shape[0], shape[1], CvType.CV_16FC(numChannel), byteBuffer)
                    "uint16" -> Mat(shape[0], shape[1], CvType.CV_16UC(numChannel), byteBuffer)
                    "uint8" -> Mat(shape[0], shape[1], CvType.CV_8UC(numChannel), byteBuffer)
                    "int16" -> Mat(shape[0], shape[1], CvType.CV_16SC(numChannel), byteBuffer)
                    "int8" -> Mat(shape[0], shape[1], CvType.CV_8SC(numChannel), byteBuffer)
                    else -> null
                }

            }
            "png" -> {
                ImageConvert.bytesToMat(bufferBytes)
            }
            else -> {
                null
            }
        }

        if (mat != null) {
            Gdx.app.postRunnable {
                val socketAsset = SocketAsset(name, mat)
                ImContext.mainAsset.update(socketAsset)
            }
        }
    }

    private fun readInt(): Int {
        val sizeBytes = ByteArray(4)
        reader.read(sizeBytes, 0, 4)
        return ByteBuffer.wrap(sizeBytes).getInt()
    }

    private fun readString(size: Int): String {
        val strBytes = ByteArray(size)
        reader.read(strBytes, 0, size)
        return String(strBytes, StandardCharsets.UTF_8)
    }
}