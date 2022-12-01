package kr.edoli.edolview.util

inline fun forever(action: () -> Unit) {
    while (true) {
        action()
    }
}
