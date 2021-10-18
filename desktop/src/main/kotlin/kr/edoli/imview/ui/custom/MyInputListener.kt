package kr.edoli.imview.ui.custom

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.utils.Timer

open class MyInputListener : InputListener() {
    var longPressKeycode = -1
    var currentLongPressSchedule: Timer.Task? = null

    override fun keyDown(event: InputEvent, keycode: Int): Boolean {
        currentLongPressSchedule?.cancel()

        longPressKeycode = keycode
        currentLongPressSchedule = Timer.schedule(object : Timer.Task() {
            override fun run() {
                keyLongDown(keycode)
            }
        }, 0.4f)
        return super.keyDown(event, keycode)
    }

    override fun keyUp(event: InputEvent, keycode: Int): Boolean {
        if (longPressKeycode == keycode) {
            currentLongPressSchedule?.cancel()
            longPressKeycode = -1
        }
        return super.keyUp(event, keycode)
    }

    open fun keyLongDown(keycode: Int) {
        if (longPressKeycode == keycode) {
            Gdx.app.postRunnable {
                keyLongDown(keycode)
            }
        }
    }
}