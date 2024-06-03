package kr.edoli.edolview.util

import java.nio.Buffer
import java.nio.ByteBuffer
import java.nio.ByteOrder

object NativeBuffer {
    private val address = Buffer::class.java.getDeclaredField("address")
    private val capacity = Buffer::class.java.getDeclaredField("capacity")

    init {
        address.isAccessible = true
        capacity.isAccessible = true
    }

    fun wrapAddress(buffer: Buffer, addr: Long, length: Int) {
        try {
            address.setLong(buffer, addr)
            capacity.setInt(buffer, length)
            buffer.clear()
        } catch (e: IllegalAccessException) {
            throw AssertionError(e);
        }
    }

    fun wrapAddressByteBuffer(addr: Long, length: Int): ByteBuffer {
        val buffer = ByteBuffer.allocateDirect(0).order(ByteOrder.nativeOrder())
        try {
            address.setLong(buffer, addr)
            capacity.setInt(buffer, length)
            buffer.clear()
        } catch (e: IllegalAccessException) {
            throw AssertionError(e);
        }
        return buffer
    }
}