package kr.edoli.imview.ui

import com.badlogic.gdx.Input
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.actions.Actions
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext
import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.ui.contextmenu.ContextMenuManager
import kr.edoli.imview.ui.custom.CustomSlider
import kr.edoli.imview.ui.res.Colors
import kr.edoli.imview.ui.res.Font
import kr.edoli.imview.ui.res.uiSkin
import kr.edoli.imview.util.ObservableList
import kr.edoli.imview.util.ObservableValue
import kr.edoli.imview.util.toColor
import kr.edoli.imview.util.toColorStr
import org.opencv.core.Rect
import tornadofx.isDouble
import tornadofx.isFloat
import kotlin.math.abs

object UIFactory {
    val tooltipManager = TooltipManager().apply {
        initialTime = 0.3f
        subsequentTime = 0.1f
        animations = false
    }
    val contextMenuManager = ContextMenuManager()

    val textButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.defaultFont
        fontColor = Colors.normal
        downFontColor = Colors.negative
        overFontColor = Colors.over
    }
    val textToggleButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.defaultFont
        fontColor = Colors.normal
        downFontColor = Colors.negative
        overFontColor = Colors.over

        up = uiSkin.getDrawable("default-round")
        down = uiSkin.getDrawable("default-round-down")
        checked = uiSkin.getDrawable("default-round-down")
    }

    val iconLabelStyle = Label.LabelStyle(Font.ioniconsFont, Colors.normal)
    val iconButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.ioniconsFont
        fontColor = Colors.normal
        downFontColor = Colors.negative
        overFontColor = Colors.over
    }
    val iconToggleButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.ioniconsFont
        fontColor = Colors.normal
        downFontColor = Colors.negative
        overFontColor = Colors.over
        checkedFontColor = Colors.accent
    }

    fun createNumberField(observable: ObservableValue<Float>): TextField {
        return object : TextField("", uiSkin) {
            override fun getPrefWidth(): Float {
                return 0f
            }
        }.apply {
            observable.subscribe {
                text = it.toString()
            }

            addListener(object : InputListener() {
                override fun keyUp(event: InputEvent, keycode: Int): Boolean {
                    if (keycode == Input.Keys.ENTER) {
                        if (text.isFloat()) {
                            observable.update(text.toFloat())
                        } else {
                            text = observable.get().toString()
                        }
                    }
                    return super.keyUp(event, keycode)
                }
            })
        }
    }


    fun createSlider(icon: String, min: Float, max: Float, stepSize: Float, observable: ObservableValue<Float>): Table {
        return Table().apply {
            add(Label(icon, iconLabelStyle)).padRight(8f)
            add(CustomSlider(min, max, stepSize, false, uiSkin).apply {
                setButton(Input.Buttons.LEFT)
                observable.subscribe { newValue ->
                    this.value = newValue
                }

                addListener(object : InputListener() {
                    override fun scrolled(event: InputEvent, x: Float, y: Float, amountX: Float, amountY: Float): Boolean {
                        when {
                            UIUtils.ctrl() -> {
                                this@apply.value = this@apply.value - stepSize * amountY
                            }
                            UIUtils.shift() -> {
                                this@apply.value = this@apply.value - stepSize * amountY * 10
                            }
                            else -> {
                                this@apply.value = this@apply.value - stepSize * amountY * 100
                            }
                        }
                        return true
                    }

                    override fun enter(event: InputEvent, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
                        stage.scrollFocus = this@apply
                    }

                    override fun exit(event: InputEvent, x: Float, y: Float, pointer: Int, toActor: Actor?) {
                        if (toActor == null || toActor != this@apply) {
                            stage.scrollFocus = null
                        }
                    }
                })

                addListener(object : ChangeListener() {
                    override fun changed(event: ChangeEvent, actor: Actor) {
                        val value = this@apply.value
                        observable.update(value)
                    }
                })
            }.contextMenu {
                addMenu("Reset value") {
                    observable.reset()
                }
                addMenu("Copy value") {
                    ClipboardUtils.putString(observable.get().toString())
                }
            }).expandX().fillX()
            add(createLabel(observable, null) {
                String.format("%.2f", abs(it))
            }.apply {
                addListener {
                    color = if (observable.get() < 0) {
                        Colors.negative
                    } else {
                        Colors.normal
                    }
                    return@addListener false
                }
            }).width(40f)
        }.tooltip("${observable.name}\n[negative]Red[] means negative")
    }

    fun <T> createSelectBox(observable: ObservableList<T>): SelectBox<T> {
        return SelectBox<T>(uiSkin).apply {
            observable.subscribe { newValue ->
                selected = newValue
                if (observable.items != items) {
                    val array = com.badlogic.gdx.utils.Array<T>()
                    observable.items.forEach { array.add(it) }
                    items = array
                }
            }

            addListener(object : ChangeListener() {
                override fun changed(event: ChangeEvent?, actor: Actor?) {
                    observable.update(selectedIndex)
                }
            })
            addListener(object : InputListener() {
                override fun enter(event: InputEvent, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
                    if (fromActor != event.target) {
                        stage.scrollFocus = event.target
                    }
                    super.enter(event, x, y, pointer, fromActor)
                }

                override fun exit(event: InputEvent, x: Float, y: Float, pointer: Int, toActor: Actor?) {
                    if (toActor != event.target) {
                        stage.scrollFocus = null
                    }
                    super.exit(event, x, y, pointer, toActor)
                }

                override fun scrolled(event: InputEvent, x: Float, y: Float, amountX: Float, amountY: Float): Boolean {
                    var nextIndex = selectedIndex + amountY.toInt()
                    if (nextIndex < 0) {
                        nextIndex += items.size
                    }
                    if (nextIndex >= items.size) {
                        nextIndex -= items.size
                    }
                    selectedIndex = nextIndex
                    event.stop()
                    return true
                }
            })
        }.tooltip(observable.name).contextMenu {
            addMenu("Reset value") {
                observable.reset()
            }
            addMenu("Copy value") {
                ClipboardUtils.putString(observable.get().toString())
            }
        }
    }

    fun createToggleIconButton(text: String, observable: ObservableValue<Boolean>) =
            createToggleTextButton(text, observable).apply {
                style = iconToggleButtonStyle
                align(Align.center)
            }.tooltip(observable.name).contextMenu {
                addMenu("Copy value") {
                    ClipboardUtils.putString(observable.get().toString())
                }
            }

    fun createToggleTextButton(text: String, observable: ObservableValue<Boolean>): TextButton {
        return TextButton(text, uiSkin.get("toggle", TextButton.TextButtonStyle::class.java)).apply {
            observable.subscribe { newValue -> isChecked = newValue }
            addListener(object : ChangeListener() {
                override fun changed(event: ChangeEvent?, actor: Actor?) {
                    observable.update(this@apply.isChecked)
                }
            })
        }.tooltip(observable.name).contextMenu {
            addMenu("Copy value") {
                ClipboardUtils.putString(observable.get().toString())
            }
        }
    }

    fun createIconButton(text: String, action: (button: Button) -> Unit) =
            createTextButton(text, action).apply {
                style = iconButtonStyle
                align(Align.center)
            }

    fun createTextButton(text: String, action: (button: Button) -> Unit): TextButton {
        return TextButton(text, uiSkin).apply {
            addListener(object : ClickListener() {
                override fun clicked(event: InputEvent?, x: Float, y: Float) {
                    action(this@apply)
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
        }.contextMenu {
            addMenu("Copy hex") {
                ClipboardUtils.putString(observable.get().toColor().toString())
            }
            addMenu("Copy color") {
                val color = observable.get()
                val imageSpec = ImContext.mainImageSpec.get()
                if (imageSpec != null) {
                    ClipboardUtils.putString("(${color.toColorStr(imageSpec.maxValue)})")
                }
            }
        }
    }

    fun createRectLabel(observable: ObservableValue<Rect>): Label {
        return createLabel(observable) { newValue -> "(${newValue.x}, ${newValue.y}, ${newValue.width}, ${newValue.height})" }
    }

    fun createPointLabel(observable: ObservableValue<Point2D>): Label {
        return createLabel(observable) { newValue -> "(${newValue.x.toInt()}, ${newValue.y.toInt()})" }
    }

    fun <T> createLabel(observable: ObservableValue<T>
                        , tooltipText: String? = observable.name
                        , text: (value: T) -> String = { it.toString() })
            : Label {
        return {
            var lastValue: T? = null
            Label("", uiSkin).apply {
                observable.subscribe { newValue ->
                    lastValue = newValue
                    setText(text(newValue))
                }
            }.tooltip(tooltipText).contextMenu {
                if (lastValue !is String) {
                    addMenu("Copy value") {
                        val value = lastValue
                        val res = checkArray(value)
                        if (res != null) {
                            ClipboardUtils.putString("[$res]")
                        } else {
                            ClipboardUtils.putString(lastValue.toString())
                        }
                    }
                }
                addMenu("Copy text") {
                    val value = lastValue
                    ClipboardUtils.putString(value?.let { text(it) } ?: "")
                }
            }
        }()

    }

    @kotlin.ExperimentalUnsignedTypes
    fun checkArray(value: Any?): String? {
        when (value) {
            is Array<*> -> {
                return value.joinToString(",")
            }
            is IntArray -> {
                return value.joinToString(",")
            }
            is FloatArray -> {
                return value.joinToString(",")
            }
            is DoubleArray -> {
                return value.joinToString(",")
            }
            is ByteArray -> {
                return value.joinToString(",")
            }
            is CharArray -> {
                return value.joinToString(",")
            }
            is ShortArray -> {
                return value.joinToString(",")
            }
            is UByteArray -> {
                return value.joinToString(",")
            }
            is LongArray -> {
                return value.joinToString(",")
            }
            is UIntArray -> {
                return value.joinToString(",")
            }
            is BooleanArray -> {
                return value.joinToString(",")
            }
            is UShortArray -> {
                return value.joinToString(",")
            }
            is ULongArray -> {
                return value.joinToString(",")
            }
            else -> return null
        }
    }
}