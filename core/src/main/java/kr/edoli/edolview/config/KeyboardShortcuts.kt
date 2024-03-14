package kr.edoli.edolview.config

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input.Keys.*
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils


object KeyboardShortcuts {

    // UI
    var UI_SCALE_DOWN = Shortcut(LEFT_BRACKET, ctrl = true)
    var UI_SCALE_UP = Shortcut(RIGHT_BRACKET, ctrl = true)
    var PRESENTATION_MODE_TOGGLE = Shortcut(P, ctrl = true)
    var RGB_TOOLTIP_TOGGLE = Shortcut(E, ctrl = true)
    var FULLSCREEN_TOGGLE = Shortcut(F11)
    var LOAD_FROM_CLIPBOARD = Shortcut(D, ctrl = true)

    // DEBUG
    var SHOW_OBSERVABLE_INFO = Shortcut(F3)
    var SHOW_DEBUG_UI = Shortcut(F4)
    var REFRESH_ASSET = Shortcut(F5)

    class Shortcut(vararg val keys: Int,
                   val ctrl: Boolean = false,
                   val shift: Boolean = false,
                   val alt: Boolean = false) {
        fun check(keycode: Int): Boolean {
            if (keycode in keys) {
                var result = true

                for (key in keys) {
                    if (!Gdx.input.isKeyPressed(key)) {
                        result = false
                    }
                }
                if (ctrl && !UIUtils.ctrl()) {
                    result = false
                }
                if (shift && !UIUtils.shift()) {
                    result = false
                }
                if (alt && !UIUtils.alt()) {
                    result = false
                }
                return result
            }

            return false
        }
    }
}