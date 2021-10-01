package kr.edoli.imview.ui.custom

import com.badlogic.gdx.Input
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.actions.Actions
import com.badlogic.gdx.scenes.scene2d.ui.Button
import com.badlogic.gdx.scenes.scene2d.ui.Container
import com.badlogic.gdx.scenes.scene2d.ui.Widget
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener

open class DropDown(
        val button: Button,
        val dropDown: Widget
) : Container<Actor>(button) {

    val stagePosition = Vector2()
    var keyboardFocusCache: Actor? = null
    var scrollFocusCache: Actor? = null

    init {
        actor.addListener(object : ClickListener() {
            var isShown = false
            override fun touchDown(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                isShown = dropDown.stage != null

                return super.touchDown(event, x, y, pointer, button)
            }
            override fun clicked(event: InputEvent, x: Float, y: Float) {
                if (!isShown) {
                    showDropDown()
                } else {
                    hideDropDown()
                }
            }
        })
    }

    override fun act(delta: Float) {
        button.isChecked = dropDown.stage != null
        super.act(delta)
    }


    val hideListener = object : InputListener() {
        override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
            val target = event.target
            return if (dropDown.isAscendantOf(target)) {
                false
            } else {
                hideDropDown()
                false
            }
        }

        override fun keyDown(event: InputEvent, keycode: Int): Boolean {
            return when (keycode) {
                Input.Keys.ESCAPE -> {
                    hideDropDown()
                    event.stop()
                    true
                }
                else -> false
            }
        }
    }

    open fun showDropDown() {
        if (stage != null) {
            stage.addActor(dropDown)

            this.localToStageCoordinates(this.stagePosition.set(0.0f, 0.0f))
            dropDown.layout()
            dropDown.pack()

            dropDown.setPosition(stagePosition.x, stagePosition.y - dropDown.height)

            stage.addCaptureListener(hideListener)

            keyboardFocusCache = stage.keyboardFocus
            scrollFocusCache = stage.scrollFocus
            stage.keyboardFocus = dropDown
            stage.scrollFocus = dropDown
        }
    }

    open fun hideDropDown() {
        if (dropDown.stage != null) {
            val prevKeyboardFocused = keyboardFocusCache
            if (prevKeyboardFocused != null) {
                stage.keyboardFocus = prevKeyboardFocused
            }
            val prevScrollFocused = scrollFocusCache
            if (prevScrollFocused != null) {
                stage.scrollFocus = prevScrollFocused
            }
            stage.removeCaptureListener(hideListener)
            dropDown.addAction(Actions.removeActor())
        }
    }


}

