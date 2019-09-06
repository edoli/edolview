package kr.edoli.imview.util

import kotlin.contracts.contract

inline fun forever(action: () -> Unit) {
    while (true) {
        action()
    }
}
