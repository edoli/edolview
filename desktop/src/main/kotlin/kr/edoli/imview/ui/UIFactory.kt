package kr.edoli.imview.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext
import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.max
import kr.edoli.imview.image.typeMax
import kr.edoli.imview.util.ObservableValue
import kr.edoli.imview.util.toColor
import kr.edoli.imview.util.toColorStr
import org.opencv.core.Rect

object UIFactory {
    val skin = Skin(Gdx.files.internal("uiskin.json"))
    val tooltipManager = TooltipManager().apply {
        initialTime = 0f
    }

    val defaultTextButtonStyle = skin.get(TextButton.TextButtonStyle::class.java)
    val toggleTextButtonStyle = skin.get("toggle", TextButton.TextButtonStyle::class.java)
    val iconButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.ioniconsFont
        fontColor = Color.WHITE
        downFontColor = Color.RED
    }
    val iconToggleButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.ioniconsFont
        fontColor = Color.WHITE
        downFontColor = Color.RED
        checkedFontColor = Color.GREEN
    }


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

            addListener(object : TextTooltip(observable.name, tooltipManager, skin) {

            })
        }
    }

    fun createToggleIconButton(text: String, observable: ObservableValue<Boolean>) =
            createToggleTextButton(text, observable).apply {
                style = iconToggleButtonStyle
                align(Align.center)
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

    fun createIconButton(text: String, action: () -> Unit) =
            createTextButton(text, action).apply {
                style = iconButtonStyle
                align(Align.center)
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

    fun createColorRect(observable: ObservableValue<DoubleArray>): ColorRect {
        return ColorRect().apply {
            observable.subscribe { newValue -> color = newValue.toColor() }
        }
    }

    fun createColorLabel(observable: ObservableValue<DoubleArray>): Label {
        return Label("", skin).apply {
            observable.subscribe { newValue ->
                ImContext.mainImageSpec.get()?.let { imageSpec ->
                    setText("(${newValue.toColorStr(imageSpec.maxValue)})")
                }
            }
        }
    }

    fun createRectLabel(observable: ObservableValue<Rect>): Label {
        return Label("", skin).apply {
            observable.subscribe { newValue ->
                setText("(${newValue.x}, ${newValue.y}, ${newValue.width}, ${newValue.height})")
            }
        }
    }

    fun createPointLabel(observable: ObservableValue<Point2D>): Label {
        return Label("", skin).apply {
            observable.subscribe { newValue ->
                setText("(${newValue.x.toInt()}, ${newValue.y.toInt()})")
            }
        }
    }
}