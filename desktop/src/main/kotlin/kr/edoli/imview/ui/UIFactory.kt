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
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.ui.contextmenu.ContextMenuManager
import kr.edoli.imview.util.ObservableValue
import kr.edoli.imview.util.format
import kr.edoli.imview.util.toColor
import kr.edoli.imview.util.toColorStr
import org.opencv.core.Rect

object UIFactory {
    val skin = Skin(Gdx.files.internal("uiskin.json"))
    val tooltipManager = TooltipManager().apply {
        initialTime = 0f
    }
    val contextMenuManager = ContextMenuManager()

    val defaultTextButtonStyle = skin.get(TextButton.TextButtonStyle::class.java)
    val toggleTextButtonStyle = skin.get("toggle", TextButton.TextButtonStyle::class.java)

    val iconLabelStyle = Label.LabelStyle(Font.ioniconsFont, Color.WHITE)
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


    fun createSlider(icon: String, min: Float, max: Float, stepSize: Float, observable: ObservableValue<Float>): Table {
        return Table().apply {
            add(Label(icon, iconLabelStyle)).padRight(8f)
            add(Slider(min, max, stepSize, false, UIFactory.skin).apply {
                observable.subscribe { newValue ->
                    this.value = newValue
                }

                addListener(object : ChangeListener() {
                    override fun changed(event: ChangeEvent, actor: Actor) {
                        val value = this@apply.value
                        observable.update(value)
                    }
                })
            })
        }.tooltip(observable.name)
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
        }.tooltip(observable.name)
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
        }.tooltip(observable.name).contextMenu {
            addMenu("Copy hex") {
                ClipboardUtils.putString(observable.get().toColor().toString())
            }
            addMenu("Copy rgba") {
                val color = observable.get().toColor()
                ClipboardUtils.putString("(${color.r}, ${color.g}, ${color.b})")
            }
            addMenu("Copy rgb") {
                val color = observable.get().toColor()
                ClipboardUtils.putString("(${color.r}, ${color.g}, ${color.b}, ${color.a})")
            }
        }
    }

    fun createColorLabel(observable: ObservableValue<DoubleArray>): Label {
        return createLabel(observable) { newValue ->
            val imageSpec = ImContext.mainImageSpec.get()
            if (imageSpec != null) {
                "(${newValue.toColorStr(imageSpec.maxValue)})"
            } else {
                ""
            }
        }
    }

    fun createRectLabel(observable: ObservableValue<Rect>): Label {
        return createLabel(observable) { newValue -> "(${newValue.x}, ${newValue.y}, ${newValue.width}, ${newValue.height})" }
    }

    fun createPointLabel(observable: ObservableValue<Point2D>): Label {
        return createLabel(observable) { newValue -> "(${newValue.x.toInt()}, ${newValue.y.toInt()})" }
    }

    fun <T> createLabel(observable: ObservableValue<T>, text: (value: T) -> String): Label {
        return {
            var lastValue: T? = null
            Label("", skin).apply {
                observable.subscribe { newValue ->
                    lastValue = newValue
                    setText(text(newValue))
                }
            }.tooltip(observable.name).contextMenu {
                addMenu("Copy value") {
                    val value = lastValue
                    val res = checkArray(value)
                    if (res != null) {
                        ClipboardUtils.putString("[$res]")
                    } else {
                        ClipboardUtils.putString(lastValue.toString())
                    }
                }
                addMenu("Copy text") {
                    val value = lastValue
                    ClipboardUtils.putString(value?.let { text(it) } ?: "")
                }
            }
        }()

    }

    fun checkArray(value: Any?): String? {
        if (value is Array<*>) {
            return value.joinToString(",")
        } else if (value is IntArray) {
            return value.joinToString(",")
        } else if (value is FloatArray) {
            return value.joinToString(",")
        } else if (value is DoubleArray) {
            return value.joinToString(",")
        } else if (value is ByteArray) {
            return value.joinToString(",")
        } else if (value is CharArray) {
            return value.joinToString(",")
        } else if (value is ShortArray) {
            return value.joinToString(",")
        } else if (value is UByteArray) {
            return value.joinToString(",")
        } else if (value is LongArray) {
            return value.joinToString(",")
        } else if (value is UIntArray) {
            return value.joinToString(",")
        } else if (value is BooleanArray) {
            return value.joinToString(",")
        } else if (value is UShortArray) {
            return value.joinToString(",")
        } else if (value is ULongArray) {
            return value.joinToString(",")
        }
        return null
    }
}