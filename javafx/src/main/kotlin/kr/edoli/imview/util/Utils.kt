package kr.edoli.imview.util

import java.util.UUID


fun generateId(): Long { return UUID.randomUUID().mostSignificantBits and java.lang.Long.MAX_VALUE }