package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Cursor
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.TextButton
import com.badlogic.gdx.scenes.scene2d.ui.TextField
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import kr.edoli.edoliui.res.Fonts
import kr.edoli.edoliui.widget.drawable.ColorDrawable
import kr.edoli.imview.res.Colors

/**
 * Created by daniel on 16. 9. 23.
 */
object UI {

    val iconButtonStyle = initStyle(TextButton.TextButtonStyle()) {
        font = Fonts.iconicFont
        fontColor = Color.WHITE
        up = ColorDrawable(Colors.buttonUp, 4f)
        over = ColorDrawable(Colors.buttonOver, 4f)
        down = ColorDrawable(Colors.buttonDown, 4f)
    }

    val textButtonStyle = initStyle(TextButton.TextButtonStyle()) {
        font = Fonts.textFont
        fontColor = Color.WHITE
        up = ColorDrawable(Colors.buttonUp)
        over = ColorDrawable(Colors.buttonOver)
        down = ColorDrawable(Colors.buttonDown)
    }

    val labelStyle = initStyle(Label.LabelStyle()) {
        font = Fonts.textFont
        fontColor = Color.WHITE
    }

    val textFieldStyle = initStyle(TextField.TextFieldStyle()) {
        font = Fonts.textFont
        fontColor = Color.WHITE
        selection = ColorDrawable(Colors.textSelect)
    }

    inline fun <T> initStyle(style: T, init: T.() -> Unit): T {
        style.init()
        return style
    }

    fun iconButton(iconCode: String) = TextButton(iconCode, iconButtonStyle).cursor(Cursor.SystemCursor.Hand)
    fun textButton(text: String) = TextButton(text, textButtonStyle).cursor(Cursor.SystemCursor.Hand)
    fun label(text: String) = Label(text, labelStyle)
    fun textField(text: String) = TextField(text, textFieldStyle).cursor(Cursor.SystemCursor.Ibeam)

}

fun <T : Actor> T.cursor(cursor: Cursor.SystemCursor): T {
    this.addListener(object : ClickListener() {
        override fun enter(event: InputEvent?, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
            super.enter(event, x, y, pointer, fromActor)
            if (isOver) {
                Gdx.graphics.setSystemCursor(cursor)
            }
        }

        override fun exit(event: InputEvent?, x: Float, y: Float, pointer: Int, toActor: Actor?) {
            if (pointer == -1) {
                Gdx.graphics.setSystemCursor(com.badlogic.gdx.graphics.Cursor.SystemCursor.Arrow)
            }
        }
    })
    return this
}

fun Actor.onClick(button: Int, onClick: (Actor) -> Unit) {
    val actor = this

    addListener(object : ClickListener(button) {
        override fun clicked(event: InputEvent, x: Float, y: Float) {
            onClick(actor)
        }
    })
}

fun Actor.onClick(onClick: (Actor) -> Unit) {
    onClick(Input.Buttons.LEFT, onClick)
}