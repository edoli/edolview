package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.ui.Skin
import com.badlogic.gdx.scenes.scene2d.ui.Slider
import com.badlogic.gdx.scenes.scene2d.ui.TextButton
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import kr.edoli.imview.util.ObservableValue

object UIFactory {
    val skin = Skin(Gdx.files.internal("uiskin.json"))

    fun createSlider(min: Float, max: Float, stepSize: Float, observable: ObservableValue<Float>): Slider {
        return Slider(min, max, stepSize, false, skin).apply {
            observable.subscribe { newValue ->
                this.value = newValue
            }

            addListener(object : ChangeListener() {
                override fun changed(event: ChangeEvent, actor: Actor) {
                    val value = this@apply.value
                    observable.update(value)
                }
            })
        }
    }

    fun createToggleTextButton(text: String, observable: ObservableValue<Boolean>): TextButton {
        return TextButton(text, skin.get("toggle", TextButton.TextButtonStyle::class.java)).apply {
            observable.subscribe { newValue -> isChecked = newValue }
            addListener(object : ChangeListener() {
                override fun changed(event: ChangeEvent?, actor: Actor?) {
                    observable.update(this@apply.isChecked)
                }
            })
        }
    }

    fun createTextButton(text: String, action: () -> Unit): TextButton {
        return TextButton(text, skin).apply {
            addListener(object : ClickListener() {
                override fun clicked(event: InputEvent?, x: Float, y: Float) {
                    action()
                }
            })
        }
    }
}