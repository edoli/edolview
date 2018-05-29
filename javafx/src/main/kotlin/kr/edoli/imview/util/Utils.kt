package kr.edoli.imview.util

/**
 * Created by daniel on 16. 9. 10.
 */

var lastId = 0

fun generateId(): Int { return lastId++ }