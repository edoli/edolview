package kr.edoli.edolview.net

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Net
import com.badlogic.gdx.net.Socket
import com.badlogic.gdx.utils.GdxRuntimeException
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.asset.SocketAsset
import java.io.BufferedInputStream
import java.nio.ByteBuffer
import java.nio.charset.StandardCharsets
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


class ImageHandler(socket: Socket) {

    private val reader = BufferedInputStream(socket.inputStream)

    fun start() {
        val nameSize = readInt()
        val extraSize = readInt()
        val bufferSize = readInt()

        val name = readString(nameSize)
        val extraStr = readString(extraSize)  // TODO: parse extra

        // Read image buffer
        val bufferBytes = ByteArray(bufferSize)

        var remain = bufferSize
        var currentPosition = 0

        while (remain > 0) {
            val recvSize = reader.read(bufferBytes, currentPosition, remain)

            remain -= recvSize
            currentPosition += recvSize
        }

        Gdx.app.postRunnable {
            ImContext.mainAsset.update(SocketAsset(name, bufferBytes))
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