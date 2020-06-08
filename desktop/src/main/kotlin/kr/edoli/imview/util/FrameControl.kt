package kr.edoli.imview.util

import kr.edoli.imview.ImContext

class FrameControl {
    var remainTime = 0.0f

    fun elapse(delta: Float) {
        val frameSpeed = ImContext.frameSpeed.get()

        if (frameSpeed == 0.0f) {
            return
        }

        remainTime -= delta

        if (remainTime < 0f) {
            remainTime = frameSpeed
            ImContext.nextImage()
        }
    }
}