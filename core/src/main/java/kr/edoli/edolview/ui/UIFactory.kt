package kr.edoli.edolview.ui

import com.badlogic.gdx.Input
import com.badlogic.gdx.graphics.Cursor
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import com.badlogic.gdx.scenes.scene2d.utils.UIUtils
import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.geom.Point2D
import kr.edoli.edolview.image.ClipboardUtils
import kr.edoli.edolview.ui.contextmenu.ContextMenuManager
import kr.edoli.edolview.ui.custom.CustomList
import kr.edoli.edolview.ui.custom.CustomSlider
import kr.edoli.edolview.ui.custom.DropDownMenu
import kr.edoli.edolview.ui.drawable.BorderedDrawable
import kr.edoli.edolview.ui.drawable.ColorDrawable
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.ui.res.Font
import kr.edoli.edolview.ui.res.uiSkin
import kr.edoli.edolview.util.*
import org.opencv.core.Rect
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
        disabledFontColor = Colors.inactive

        up = BorderedDrawable(Colors.background, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
        over = BorderedDrawable(Colors.backgroundOver, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
        down = BorderedDrawable(Colors.backgroundDown, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
    }

    val textToggleButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.defaultFont
        fontColor = Colors.normal
        downFontColor = Colors.negative
        disabledFontColor = Colors.inactive

        up = BorderedDrawable(Colors.background, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
        over = BorderedDrawable(Colors.backgroundOver, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
        down = BorderedDrawable(Colors.backgroundDown, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
        checked = BorderedDrawable(Colors.accentDarkSemi, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
        checkedOver = BorderedDrawable(Colors.accentSemi, Colors.backgroundBorder).apply { pad(2f, 6f, 2f, 6f) }
    }

    val iconLabelStyle = Label.LabelStyle(Font.ioniconsFont, Colors.normal)
    val iconButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.ioniconsFont
        fontColor = Colors.normal
        downFontColor = Colors.negative
        overFontColor = Colors.over
        disabledFontColor = Colors.inactive
    }
    val iconToggleButtonStyle = TextButton.TextButtonStyle().apply {
        font = Font.ioniconsFont
        fontColor = Colors.normal
        downFontColor = Colors.negative
        overFontColor = Colors.over
        checkedFontColor = Colors.accent
        disabledFontColor = Colors.inactive
    }

    val listStyle = List.ListStyle().apply {
        font = Font.defaultFont
        fontColorSelected = Colors.normal
        fontColorUnselected = Colors.normal

        background = BorderedDrawable(Colors.backgroundDark, Colors.backgroundBorder)
        over = ColorDrawable(Colors.backgroundOver).apply { pad(2f, 6f, 2f, 6f) }
        down = ColorDrawable(Colors.backgroundDown).apply { pad(2f, 6f, 2f, 6f) }
        selection = ColorDrawable(Colors.accent).apply { pad(2f, 6f, 2f, 6f) }
    }

    fun <T> createField(
        observable: ObservableValue<T>, strToValue: (String) -> T, valueToString: (T) -> String,
        checkValid: (String) -> Boolean
    ): TextField {
        val textField = object : TextField("", uiSkin) {
            override fun getPrefWidth(): Float {
                return 0f
            }

            override fun getPrefHeight(): Float {
                return 24f
            }
        }
        return textField.apply {
            observable.subscribe(this@UIFactory, "Double binding") {
                text = valueToString(it)
            }

            addListener(object : InputListener() {
                override fun keyUp(event: InputEvent, keycode: Int): Boolean {
                    if (keycode == Input.Keys.ENTER || keycode == Input.Keys.NUMPAD_ENTER) {
                        if (checkValid(text)) {
                            observable.update(strToValue(text))
                        } else {
                            text = valueToString(observable.get())
                        }
                    } else if (keycode == Input.Keys.ESCAPE) {
                        text = valueToString(observable.get())
                        stage.keyboardFocus = null
                    }
                    color = if (text != valueToString(observable.get())) {
                        Colors.accent
                    } else {
                        Colors.normal
                    }
                    return super.keyUp(event, keycode)
                }
            })
        }.cursor(Cursor.SystemCursor.Ibeam).contextMenu {
            addMenu("Reset value") {
                observable.reset()
            }
            addMenu("Copy value") {
                ClipboardUtils.putString(observable.get().toString())
            }
            addMenu("Copy text") {
                ClipboardUtils.putString(textField.text)
            }
        }.presentation()
    }

    fun createIntField(observable: ObservableValue<Int>) = createField(observable, {
        it.toInt()
    }, {
        it.toString()
    }, {
        it.isInt()
    })

    fun createFloatField(observable: ObservableValue<Float>) = createField(observable, {
        it.toFloat()
    }, {
        it.toString()
    }, {
        it.isFloat()
    })

    fun createRectField(observable: ObservableValue<Rect>) = createField(observable, { str ->
        if (str.contains(":")) {
            val numbers = str.replace("[()\\[\\]]".toRegex(), "").replace(":", ",").split(",").map {
                println(it)
                it.trim().toInt()
            }
            Rect(numbers[2], numbers[0], numbers[3] - numbers[2], numbers[1] - numbers[0])
        } else {
            val numbers = str.replace("(", "").replace(")", "").split(",").map {
                it.trim().toInt()
            }
            Rect(numbers[0], numbers[1], numbers[2], numbers[3])
        }
    }, { newValue ->
        "(${newValue.x}, ${newValue.y}, ${newValue.width}, ${newValue.height})"
    }, { str ->
        if (str.contains(":")) {
            str.replace("[()\\[\\]]".toRegex(), "").replace(":", ",").split(",").map {
                it.trim().isInt()
            }.reduce { acc, b -> acc && b }
        } else {
            str.replace("(", "").replace(")", "").split(",").map {
                it.trim().isInt()
            }.reduce { acc, b -> acc && b }
        }
    }).contextMenu {
        addMenu("Copy numpy indexing") {
            val rect = observable.get()

            val text = "${rect.y}:${rect.y + rect.height}, ${rect.x}:${rect.x + rect.width}"
            ClipboardUtils.putString(text)
        }
    }

    fun createSlider(icon: String, min: Float, max: Float, stepSize: Float, observable: ObservableValue<Float>) =
        Table().apply {
            add(Label(icon, iconLabelStyle)).padRight(8f)
            add(CustomSlider(min, max, stepSize, false, uiSkin).apply {
                val slider = this@apply
                setButton(Input.Buttons.LEFT)
                defaultValue = observable.initValue
                observable.subscribe(this@UIFactory, "Double binding") { newValue ->
                    this.value = newValue
                }

                addListener(object : InputListener() {
                    override fun scrolled(
                        event: InputEvent,
                        x: Float,
                        y: Float,
                        amountX: Float,
                        amountY: Float
                    ): Boolean {
                        when {
                            UIUtils.ctrl() -> {
                                slider.value -= stepSize * amountY
                            }

                            UIUtils.shift() -> {
                                slider.value -= stepSize * amountY * 10
                            }

                            else -> {
                                slider.value -= stepSize * amountY * 100
                            }
                        }
                        return true
                    }

                    override fun enter(event: InputEvent, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
                        stage.scrollFocus = slider
                    }

                    override fun exit(event: InputEvent, x: Float, y: Float, pointer: Int, toActor: Actor?) {
                        if (toActor == null || toActor != slider) {
                            stage.scrollFocus = null
                        }
                    }
                })

                addListener(object : ChangeListener() {
                    override fun changed(event: ChangeEvent, actor: Actor) {
                        val value = slider.value
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
                observable.subscribe(this@UIFactory, "Double binding") {
                    color = if (it < 0) {
                        Colors.negative
                    } else {
                        Colors.normal
                    }
                }
            }).width(40f)
        }.tooltip("${observable.name}\n[negative]Red[] means negative")

    fun <T> createDropdownMenu(
        name: String,
        observable: ObservableList<T>,
        textFunc: (T) -> String,
        onSelected: (T) -> Unit
    ) = DropDownMenu(
            createTextButton(name).apply { style = textToggleButtonStyle },
            listStyle, textFunc, onSelected
        ).apply {
            observable.subscribe(this@UIFactory, "Double binding") { newValue, _ ->
                if (newValue != list.items) {
                    val array = com.badlogic.gdx.utils.Array<T>()
                    newValue.forEach { array.add(it) }
                    list.setItems(array)
                    list.selection.clear()
                    hideDropDown()
                }
            }
        }

    fun <T> createList(
        observable: ObservableList<T>,
        textFunc: (T) -> String,
        onSelected: (T?) -> Unit
    ) = CustomList(uiSkin, textFunc).apply {
            observable.subscribe(this@UIFactory, "Double binding") { items, newValue ->
                val array = com.badlogic.gdx.utils.Array<T>()
                items.forEach { array.add(it) }
                setItems(array)
                if (selected != newValue) {
                    selected = newValue
                }
            }

            addListener(object : ChangeListener() {
                override fun changed(event: ChangeEvent?, actor: Actor?) {
                    if (observable.currentIndex != selectedIndex) {
                        observable.update(selectedIndex)
                    }
                    onSelected(selected)
                }
            })
        }

    fun <T> createSelectBox(observable: ObservableList<T>): SelectBox<T> =
        SelectBox<T>(uiSkin).apply {
            observable.subscribeValue(this@UIFactory, "Double binding") { newValue ->
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
                    var nextIndex = (selectedIndex + amountY.toInt()) % items.size
                    if (nextIndex < 0) {
                        nextIndex += items.size
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

    fun createIcon(text: String) =
        Label(text, uiSkin).apply {
            style = iconLabelStyle
        }

    fun createToggleIconButton(text: String, observable: ObservableValue<Boolean>) =
            createToggleTextButton(text, observable).apply {
                style = iconToggleButtonStyle
                align(Align.center)
            }

    fun createToggleTextButton(text: String, observable: ObservableValue<Boolean>) =
        TextButton(text, textToggleButtonStyle).apply {
            observable.subscribe(this@UIFactory, "Double binding") { newValue -> isChecked = newValue }
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

    fun createIconButton(text: String, action: ((button: Button) -> Unit)? = null) =
        createTextButton(text, action).apply {
            style = iconButtonStyle
            align(Align.center)
        }

    fun createTextButton(text: String, action: ((button: Button) -> Unit)? = null) =
        TextButton(text, textButtonStyle).apply {
            if (action != null) {
                addListener(object : ClickListener() {
                    override fun clicked(event: InputEvent?, x: Float, y: Float) {
                        action(this@apply)
                    }
                })
            }
        }

    fun createColorRect(observable: ObservableValue<DoubleArray>) =
        ColorRect().apply {
            observable.subscribe(this@UIFactory, "Double binding") { newValue -> color = newValue.toColor() }
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
        }.presentation()

    fun createColorLabel(observable: ObservableValue<DoubleArray>) =
        createLabel(observable) { newValue ->
            val imageSpec = ImContext.mainImageSpec.get()
            if (imageSpec != null) {
                "(${newValue.toColorStr(imageSpec)})"
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
                    ClipboardUtils.putString("(${color.toColorStr(imageSpec)})")
                }
            }
        }

    fun createRectLabel(observable: ObservableValue<Rect>) =
        createLabel(observable) { newValue -> "(${newValue.x}, ${newValue.y}, ${newValue.width}, ${newValue.height})" }

    fun createPointLabel(observable: ObservableValue<Point2D>) =
        createLabel(observable) { newValue -> "(${newValue.x.toInt()}, ${newValue.y.toInt()})" }

    fun createNumberLabel(observable: ObservableValue<Double>) =
        createLabel(observable, textFormat = { it.format(2) })

    @OptIn(ExperimentalUnsignedTypes::class)
    fun <T> createLabel(observable: ObservableValue<T>, tooltipText: String? = observable.name, textFormat: (value: T) -> String = { it.toString() })
            : Label {
        var lastValue: T? = null
        return Label("", uiSkin).apply {
            observable.subscribe(this@UIFactory, "Double binding") { newValue ->
                lastValue = newValue
                setText(textFormat(newValue))
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
                ClipboardUtils.putString(value?.let<T, String> { textFormat(it) } ?: "")
            }
        }.presentation()
    }

    @ExperimentalUnsignedTypes
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