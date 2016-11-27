package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Cursor
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import kr.edoli.edoliui.res.Fonts
import kr.edoli.edoliui.widget.drawable.ColorDrawable
import kr.edoli.imview.res.Colors
import java.util.*

/**
 * Created by daniel on 16. 9. 23.
 */
object UI {

    val round = 2f

    enum class FontType {
        Text, Icon
    }

    data class ButtonStyleDesc(
            val type: FontType,
            val checkable: Boolean,
            val size: Int
    )

    val buttonStyleMap = HashMap<ButtonStyleDesc, TextButton.TextButtonStyle>()

    fun buttonStyle(desc: ButtonStyleDesc): TextButton.TextButtonStyle {
        if (buttonStyleMap.containsKey(desc)) {
            return buttonStyleMap[desc] as TextButton.TextButtonStyle
        }
        val style = initStyle(TextButton.TextButtonStyle()) {
            font = if (desc.type == FontType.Icon) Fonts.icon(desc.size) else Fonts.text(desc.size)
            fontColor = Color.WHITE
            overFontColor = Color.valueOf("#5bc0eb")
            if (desc.checkable) {
                checkedFontColor = Color.valueOf("#fde74c")
            }
            disabledFontColor= Color.GRAY
        }
        buttonStyleMap.put(desc, style)
        return style
    }


    val labelStyle = initStyle(Label.LabelStyle()) {
        font = Fonts.text(16)
        fontColor = Color.WHITE
    }

    val textFieldStyle = initStyle(TextField.TextFieldStyle()) {
        font = Fonts.text(16)
        fontColor = Color.WHITE
        selection = ColorDrawable(Colors.textSelect)
    }

    inline fun <T> initStyle(style: T, init: T.() -> Unit): T {
        style.init()
        return style
    }

    fun iconButton(iconCode: String, checkable: Boolean = false, size: Int = 24) =
            TextButton(iconCode, buttonStyle(ButtonStyleDesc(FontType.Icon, checkable, size)))
                    .cursor(Cursor.SystemCursor.Hand)
    fun textButton(text: String, checkable: Boolean = false, size: Int = 24) =
            TextButton(text, buttonStyle(ButtonStyleDesc(FontType.Icon, checkable, size)))
                    .cursor(Cursor.SystemCursor.Hand)
    fun label(text: String) = Label(text, labelStyle)
    fun textField(text: String) = TextField(text, textFieldStyle).cursor(Cursor.SystemCursor.Ibeam)

    fun optionTable(vararg buttons: Button): Table {
        val table = Table()

        val border = Image(ColorDrawable("#ffffff"))

        buttons.forEach { table.add(it).size(48f) }
        table.row()
        table.add(border).colspan(buttons.size).height(2f).expandX().fillX()

        return table
    }
}

fun <T : Actor> T.cursor(cursor: Cursor.SystemCursor): T {
    this.addListener(object : ClickListener() {
        override fun enter(event: InputEvent?, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
            super.enter(event, x, y, pointer, fromActor)
            if (isOver) {
                if (this@cursor is Button && !this@cursor.isDisabled || this@cursor !is Button) {
                    Gdx.graphics.setSystemCursor(cursor)
                }
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

fun Actor.onClick(button: Int, onClick: (Actor) -> Unit): Actor {
    val actor = this

    addListener(object : ClickListener(button) {

        override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
            event.stop()
            return super.touchDown(event, x, y, pointer, button)
        }

        override fun clicked(event: InputEvent, x: Float, y: Float) {
            onClick(actor)
        }
    })

    return actor
}

fun Actor.onClick(onClick: (Actor) -> Unit): Actor {
    return onClick(Input.Buttons.LEFT, onClick)
}