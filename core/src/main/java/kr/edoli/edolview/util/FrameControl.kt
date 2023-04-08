package kr.edoli.edolview.util

import kr.edoli.edolview.ImContext

class FrameControl {
    var remainTime = 0.0f

    fun elapse(delta: Float) {
        val frameSpeed = ImContext.frameSpeed.get()

        if (frameSpeed == 0.0f) {
            return
        }

        remainTime -= delta

        if (remainTime < 0f) {
            remainTime = 1.0f / frameSpeed
            ImContext.mainAssetNavigator.update(1)
        }
    }
}