package kr.edoli.imview.util

inline fun forever(action: () -> Unit) {
    while (true) {
        action()
    }
}
