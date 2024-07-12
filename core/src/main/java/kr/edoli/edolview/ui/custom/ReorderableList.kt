import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.g2d.BitmapFont
import com.badlogic.gdx.graphics.g2d.GlyphLayout
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.*
import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.ui.TextButton.TextButtonStyle
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener
import com.badlogic.gdx.scenes.scene2d.utils.DragAndDrop
import com.badlogic.gdx.scenes.scene2d.utils.Drawable
import com.badlogic.gdx.utils.Array
import kr.edoli.edolview.asset.Asset
import kr.edoli.edolview.util.ObservableList
import kotlin.collections.firstOrNull
import kotlin.collections.forEachIndexed
import kotlin.collections.withIndex

class ReorderableList<T>(val observableList: ObservableList<T>, val listStyle: List.ListStyle, val textButtonStyle: TextButtonStyle, val textFunc: (T) -> String) : Table() {
    private val actors = Array<Actor>()
    private val dragAndDrop = DragAndDrop()
    private var dragHighlight: Actor? = null
    private val tempCoords = Vector2()

    val selectedIndex: Int
        get() = observableList.currentIndex

    var selected: T?
        get() = if (selectedIndex >= 0 && selectedIndex < observableList.size) observableList[selectedIndex] else null
        set(value) {
            val index = if (value != null) observableList.items.indexOf(value) else -1
            updateSelection(index)
        }

    init {
        background = listStyle.background
        defaults().height(24f)
        setupDragAndDrop()
    }

    fun refreshList() {
        actors.clear()
        clearChildren()
        for (item in observableList.items.withIndex()) {
            val actor = createActor(item.value)
            val removeButton = TextButton("X", textButtonStyle)
            removeButton.addListener(object : ClickListener() {
                override fun clicked(event: InputEvent, x: Float, y: Float) {
                    observableList.update {
                        val value = (it as ArrayList).removeAt(item.index)
                        if (value is Asset) {
                            value.dispose()
                        }
                        it
                    }
                }
            })
            add(actor).growX()
            add(removeButton).row()
            actors.add(actor)
        }

        val index = selectedIndex
        actors.forEachIndexed { i, actor ->
            (actor as ReorderableList<*>.DynamicWidthTextWidget).apply {
                background = if (i == index) listStyle.selection else null
                fontColor = if (i == index) listStyle.fontColorSelected else listStyle.fontColorUnselected
            }
        }
    }

    private fun createActor(item: T): Widget {
        return DynamicWidthTextWidget(textFunc(item), listStyle.font, listStyle.fontColorUnselected)
    }

    private fun updateSelection(index: Int) {
        if (index != selectedIndex) {
            observableList.update(index)
        }
    }

    private fun setupDragAndDrop() {
        dragAndDrop.addSource(object : DragAndDrop.Source(this) {
            override fun dragStart(event: InputEvent, x: Float, y: Float, pointer: Int): DragAndDrop.Payload? {
                val index = getItemIndexAt(y)
                if (index == -1) return null

                return DragAndDrop.Payload().apply {
                    `object` = index
                    val dragActor = DynamicWidthTextWidget(textFunc(observableList.items[index]), listStyle.font, listStyle.fontColorUnselected)
                    dragActor.touchable = Touchable.disabled
                    dragActor.setSize(150f, 24f)
                    dragActor.background = listStyle.background
                    this.dragActor = dragActor
                }
            }
        })

        dragAndDrop.addTarget(object : DragAndDrop.Target(this) {
            override fun drag(source: DragAndDrop.Source, payload: DragAndDrop.Payload, x: Float, y: Float, pointer: Int): Boolean {
                val index = getItemIndexAt(y)
                showDragHighlight(index)
                return true
            }

            override fun drop(source: DragAndDrop.Source, payload: DragAndDrop.Payload, x: Float, y: Float, pointer: Int) {
                val sourceIndex = payload.`object` as Int
                val targetIndex = getItemIndexAt(y)

                observableList.changeOrder(sourceIndex, targetIndex)

                hideDragHighlight()
            }

            override fun reset(source: DragAndDrop.Source?, payload: DragAndDrop.Payload?) {
                hideDragHighlight()
            }
        })
    }

    private fun getItemIndexAt(y: Float): Int {
        tempCoords.set(0f, y)
        val itemHeight = actors.firstOrNull()?.height ?: return -1
        val index = ((actors.size * itemHeight - tempCoords.y) / itemHeight).toInt()
        return index.coerceIn(0, actors.size - 1)
    }

    private fun showDragHighlight(index: Int) {
        hideDragHighlight()
        dragHighlight = Image(listStyle.selection).apply {
            color.a = 0.5f
        }
        addActor(dragHighlight)
        dragHighlight?.setBounds(0f, this.height - (index + 1) * (actors.firstOrNull()?.height ?: 0f) - 2f, width, 4f)
    }

    private fun hideDragHighlight() {
        dragHighlight?.remove()
        dragHighlight = null
    }

    private inner class DynamicWidthTextWidget(
        private val text: String,
        private val font: BitmapFont,
        var fontColor: com.badlogic.gdx.graphics.Color
    ) : Widget() {
        private val glyphLayout = GlyphLayout()
        var background: Drawable? = null

        init {
            addListener(object : ClickListener() {
                override fun clicked(event: InputEvent?, x: Float, y: Float) {
                    val index = actors.indexOf(this@DynamicWidthTextWidget, true)
                    updateSelection(index)
                }
            })
        }

        override fun draw(batch: Batch, parentAlpha: Float) {
            background?.draw(batch, x, y, width, height)

            val color = batch.color
            batch.setColor(fontColor.r, fontColor.g, fontColor.b, fontColor.a * parentAlpha)

            glyphLayout.setText(font, text)
            val textWidth = glyphLayout.width
            val textHeight = glyphLayout.height

            if (textWidth <= width) {
                font.draw(batch, text, x, y + height / 2 + textHeight / 2)
            } else {
                val ellipsis = "..."
                glyphLayout.setText(font, ellipsis)

                var truncatedText = text
                while (truncatedText.length > 1) {
                    truncatedText = truncatedText.substring(1)
                    glyphLayout.setText(font, ellipsis + truncatedText)
                    if (glyphLayout.width <= width) {
                        font.draw(batch, ellipsis + truncatedText, x, y + height / 2 + textHeight / 2)
                        break
                    }
                }
            }

            batch.color = color
        }
    }
}