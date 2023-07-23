package kr.edoli.edolview.util;

import java.io.ByteArrayOutputStream

class CustomByteArrayOutputStream(size: Int): ByteArrayOutputStream(size) {

    fun getCount(): Int {
        return count
    }

    fun getBuf(): ByteArray? {
        return buf
    }
}