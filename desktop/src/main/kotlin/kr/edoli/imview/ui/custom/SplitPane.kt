/*******************************************************************************
 * Copyright 2011 See AUTHORS file.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package kr.edoli.imview.ui.custom

import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.math.Rectangle
import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.ui.Skin
import com.badlogic.gdx.scenes.scene2d.ui.WidgetGroup
import com.badlogic.gdx.scenes.scene2d.utils.Drawable
import com.badlogic.gdx.scenes.scene2d.utils.Layout
import com.badlogic.gdx.scenes.scene2d.utils.ScissorStack
import com.badlogic.gdx.utils.GdxRuntimeException
import kr.edoli.imview.ui.custom.SplitPane.SplitPaneStyle
import kotlin.math.max
import kotlin.math.min

/** A container that contains two widgets and is divided either horizontally or vertically. The user may resize the widgets. The
 * child widgets are always sized to fill their side of the SplitPane.
 *
 *
 * Minimum and maximum split amounts can be set to limit the motion of the resizing handle. The handle position is also prevented
 * from shrinking the children below their minimum sizes. If these limits over-constrain the handle, it will be locked and placed
 * at an averaged location, resulting in cropped children. The minimum child size can be ignored (allowing dynamic cropping) by
 * wrapping the child in a [Container] with a minimum size of 0 and [fill()][Container.fill] set, or by
 * overriding [.clampSplitAmount].
 *
 *
 * The preferred size of a SplitPane is that of the child widgets and the size of the [SplitPaneStyle.handle]. The widgets
 * are sized depending on the SplitPane size and the [split position][.setSplitAmount].
 * @author mzechner
 * @author Nathan Sweet
 */
class SplitPane
/** @param firstWidget May be null.
 * @param secondWidget May be null.
 */
(firstWidget: Actor?, secondWidget: Actor?, internal var vertical: Boolean, style: SplitPaneStyle) : WidgetGroup() {
    internal var style: SplitPaneStyle = style
    private var firstWidget: Actor? = null
    private var secondWidget: Actor? = null
    internal var controlledSize = 0f
    internal var minAmount: Float = 0.toFloat()
    internal var maxAmount = 1f

    private val firstWidgetBounds = Rectangle()
    private val secondWidgetBounds = Rectangle()
    internal var handleBounds = Rectangle()
    var isCursorOverHandle: Boolean = false
        internal set
    private val tempScissors = Rectangle()

    internal var lastPoint = Vector2()
    internal var handlePosition = Vector2()

    var isCollapsed = false
    var onSplitChanged: ((dragPos: Float) -> Unit)? = null

    var isVertical: Boolean
        get() = vertical
        set(vertical) {
            if (this.vertical == vertical)
                return
            this.vertical = vertical
            invalidateHierarchy()
        }

    var minSplitAmount: Float
        get() = minAmount
        set(minAmount) {
            if (minAmount < 0 || minAmount > 1) throw GdxRuntimeException("minAmount has to be >= 0 and <= 1")
            this.minAmount = minAmount
        }

    var maxSplitAmount: Float
        get() = maxAmount
        set(maxAmount) {
            if (maxAmount < 0 || maxAmount > 1) throw GdxRuntimeException("maxAmount has to be >= 0 and <= 1")
            this.maxAmount = maxAmount
        }

    /** @param firstWidget May be null.
     * @param secondWidget May be null.
     */
    @JvmOverloads
    constructor(firstWidget: Actor?, secondWidget: Actor?, vertical: Boolean, skin: Skin, styleName: String = "default-" + if (vertical) "vertical" else "horizontal") : this(firstWidget, secondWidget, vertical, skin.get<SplitPaneStyle>(styleName, SplitPaneStyle::class.java)) {
    }

    init {
        setStyle(style)
        setFirstWidget(firstWidget)
        setSecondWidget(secondWidget)
        setSize(prefWidth, prefHeight)
        initialize()
    }

    private fun initialize() {
        addListener(object : InputListener() {
            internal var draggingPointer = -1

            override fun touchDown(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                if (draggingPointer != -1) return false
                if (pointer == 0 && button != 0) return false
                if (handleBounds.contains(x, y)) {
                    draggingPointer = pointer
                    lastPoint.set(x, y)
                    handlePosition.set(handleBounds.x, handleBounds.y)
                    return true
                }
                return false
            }

            override fun touchUp(event: InputEvent?, x: Float, y: Float, pointer: Int, button: Int) {
                if (pointer == draggingPointer) draggingPointer = -1
            }

            override fun touchDragged(event: InputEvent?, x: Float, y: Float, pointer: Int) {
                if (pointer != draggingPointer) return

                val handle = style.handle
                if (!vertical) {
                    val delta = x - lastPoint.x
                    val availWidth = width - handle.minWidth
                    var dragX = handlePosition.x + delta
                    handlePosition.x = dragX
                    dragX = Math.max(0f, dragX)
                    dragX = Math.min(availWidth, dragX)
                    controlledSize = availWidth - dragX
                    lastPoint.set(x, y)
                    onSplitChanged?.invoke(dragX)
                } else {
                    val delta = y - lastPoint.y
                    val availHeight = height - handle.minHeight
                    var dragY = handlePosition.y + delta
                    handlePosition.y = dragY
                    dragY = Math.max(0f, dragY)
                    dragY = Math.min(availHeight, dragY)
                    controlledSize = availHeight - dragY
                    lastPoint.set(x, y)
                    onSplitChanged?.invoke(dragY)
                }
                invalidate()
            }

            override fun mouseMoved(event: InputEvent?, x: Float, y: Float): Boolean {
                isCursorOverHandle = handleBounds.contains(x, y)
                return false
            }
        })
    }

    fun setStyle(style: SplitPaneStyle) {
        this.style = style
        invalidateHierarchy()
    }

    /** Returns the split pane's style. Modifying the returned style may not have an effect until [.setStyle]
     * is called.  */
    fun getStyle(): SplitPaneStyle {
        return style
    }

    override fun layout() {
        clampSplitAmount()
        if (!vertical)
            calculateHorizBoundsAndPositions()
        else
            calculateVertBoundsAndPositions()

        val firstWidget = this.firstWidget
        if (firstWidget != null) {
            val firstWidgetBounds = this.firstWidgetBounds
            firstWidget.setBounds(firstWidgetBounds.x, firstWidgetBounds.y, firstWidgetBounds.width, firstWidgetBounds.height)
            if (firstWidget is Layout) (firstWidget as Layout).validate()
        }
        val secondWidget = this.secondWidget
        if (secondWidget != null) {
            val secondWidgetBounds = this.secondWidgetBounds
            secondWidget.setBounds(secondWidgetBounds.x, secondWidgetBounds.y, secondWidgetBounds.width, secondWidgetBounds.height)
            if (secondWidget is Layout) (secondWidget as Layout).validate()
        }
    }

    override fun getPrefWidth(): Float {
        val first = if (firstWidget == null)
            0f
        else
            if (firstWidget is Layout) (firstWidget as Layout).prefWidth else firstWidget!!.width
        val second = if (secondWidget == null)
            0f
        else
            if (secondWidget is Layout) (secondWidget as Layout).prefWidth else secondWidget!!.width
        return if (vertical) max(first, second) else first + style.handle.minWidth + second
    }

    override fun getPrefHeight(): Float {
        val first = if (firstWidget == null)
            0f
        else
            if (firstWidget is Layout) (firstWidget as Layout).prefHeight else firstWidget!!.height
        val second = if (secondWidget == null)
            0f
        else
            if (secondWidget is Layout) (secondWidget as Layout).prefHeight else secondWidget!!.height
        return if (!vertical) max(first, second) else first + style.handle.minHeight + second
    }

    override fun getMinWidth(): Float {
        val first = if (firstWidget is Layout) (firstWidget as Layout).minWidth else 0f
        val second = if (secondWidget is Layout) (secondWidget as Layout).minWidth else 0f
        return if (vertical) max(first, second) else first + style.handle.minWidth + second
    }

    override fun getMinHeight(): Float {
        val first = if (firstWidget is Layout) (firstWidget as Layout).minHeight else 0f
        val second = if (secondWidget is Layout) (secondWidget as Layout).minHeight else 0f
        return if (!vertical) max(first, second) else first + style.handle.minHeight + second
    }

    private fun calculateHorizBoundsAndPositions() {
        val handle = style.handle

        val height = height

        val availWidth = width - handle.minWidth
        val rightAreaWidth = controlledSize
        val leftAreaWidth = availWidth - rightAreaWidth
        val handleWidth = handle.minWidth

        firstWidgetBounds.set(0f, 0f, leftAreaWidth, height)
        secondWidgetBounds.set(leftAreaWidth + handleWidth, 0f, rightAreaWidth, height)
        handleBounds.set(leftAreaWidth, 0f, handleWidth, height)
    }

    private fun calculateVertBoundsAndPositions() {
        val handle = style.handle

        val width = width
        val height = height

        val availHeight = height - handle.minHeight
        val bottomAreaHeight = controlledSize
        val topAreaHeight = availHeight - bottomAreaHeight
        val handleHeight = handle.minHeight

        firstWidgetBounds.set(0f, height - topAreaHeight, width, topAreaHeight)
        secondWidgetBounds.set(0f, 0f, width, bottomAreaHeight)
        handleBounds.set(0f, bottomAreaHeight, width, handleHeight)
    }

    override fun draw(batch: Batch?, parentAlpha: Float) {
        val stage = stage ?: return

        validate()

        val color = color
        val alpha = color.a * parentAlpha

        applyTransform(batch!!, computeTransform())
        if (firstWidget != null && firstWidget!!.isVisible) {
            batch.flush()
            stage.calculateScissors(firstWidgetBounds, tempScissors)
            if (ScissorStack.pushScissors(tempScissors)) {
                firstWidget!!.draw(batch, alpha)
                batch.flush()
                ScissorStack.popScissors()
            }
        }
        if (secondWidget != null && secondWidget!!.isVisible) {
            batch.flush()
            stage.calculateScissors(secondWidgetBounds, tempScissors)
            if (ScissorStack.pushScissors(tempScissors)) {
                secondWidget!!.draw(batch, alpha)
                batch.flush()
                ScissorStack.popScissors()
            }
        }
        batch.setColor(color.r, color.g, color.b, alpha)
        style.handle.draw(batch, handleBounds.x, handleBounds.y, handleBounds.width, handleBounds.height)
        resetTransform(batch)
    }

    /** @param splitAmount The split amount between the min and max amount. This parameter is clamped during
     * layout. See [.clampSplitAmount].
     */
    fun setSplitAmount(splitAmount: Float) {
        val handle = style.handle
        if (!isVertical) {
            val availWidth = width - handle.minWidth
            controlledSize = availWidth * (1 - splitAmount)
        } else {
            val availHeight = height - handle.minHeight
            controlledSize = availHeight * (1 - splitAmount)
        }
        invalidate()
    }

    fun getSplitAmount(): Float {
        val handle = style.handle
        if (!isVertical) {
            val availWidth = width - handle.minWidth
            return (availWidth - controlledSize) / availWidth
        } else {
            val availHeight = height - handle.minHeight
            return (availHeight - controlledSize) / availHeight
        }
    }

    /** Called during layout to clamp the [.splitAmount] within the set limits. By default it imposes the limits of the
     * [min amount][.getMinSplitAmount], [max amount][.getMaxSplitAmount], and min sizes of the children. This
     * method is internally called in response to layout, so it should not call [.invalidate].  */
    protected fun clampSplitAmount() {
        var effectiveMinAmount = minAmount
        var effectiveMaxAmount = maxAmount

        if (isCollapsed) {
            controlledSize = 0f
        } else {
            if (vertical) {
                val availableHeight = height - style.handle.minHeight
                if (secondWidget is Layout) {
                    controlledSize = max(controlledSize, (secondWidget as Layout).minHeight)
                }
            } else {
                val availableWidth = width - style.handle.minWidth
                if (secondWidget is Layout) {
                    controlledSize = max(controlledSize, (secondWidget as Layout).minWidth)
                }
            }
        }
    }

    /** @param widget May be null.
     */
    fun setFirstWidget(widget: Actor?) {
        if (firstWidget != null) super.removeActor(firstWidget)
        firstWidget = widget
        if (widget != null) super.addActor(widget)
        invalidate()
    }

    /** @param widget May be null.
     */
    fun setSecondWidget(widget: Actor?) {
        if (secondWidget != null) super.removeActor(secondWidget)
        secondWidget = widget
        if (widget != null) super.addActor(widget)
        invalidate()
    }

    override fun addActor(actor: Actor) {
        throw UnsupportedOperationException("Use SplitPane#setWidget.")
    }

    override fun addActorAt(index: Int, actor: Actor) {
        throw UnsupportedOperationException("Use SplitPane#setWidget.")
    }

    override fun addActorBefore(actorBefore: Actor, actor: Actor) {
        throw UnsupportedOperationException("Use SplitPane#setWidget.")
    }

    override fun removeActor(actor: Actor?): Boolean {
        requireNotNull(actor) { "actor cannot be null." }
        if (actor === firstWidget) {
            setFirstWidget(null)
            return true
        }
        if (actor === secondWidget) {
            setSecondWidget(null)
            return true
        }
        return true
    }

    override fun removeActor(actor: Actor?, unfocus: Boolean): Boolean {
        requireNotNull(actor) { "actor cannot be null." }
        if (actor === firstWidget) {
            super.removeActor(actor, unfocus)
            firstWidget = null
            invalidate()
            return true
        }
        if (actor === secondWidget) {
            super.removeActor(actor, unfocus)
            secondWidget = null
            invalidate()
            return true
        }
        return false
    }

    override fun hit(x: Float, y: Float, touchable: Boolean): Actor? {
        val target = super.hit(x, y, touchable)

        if (target == this && !handleBounds.contains(x, y)) {
            return null
        }
        return target
    }

    /** The style for a splitpane, see [SplitPane].
     * @author mzechner
     * @author Nathan Sweet
     */
    class SplitPaneStyle {
        lateinit var handle: Drawable

        constructor() {}

        constructor(handle: Drawable) {
            this.handle = handle
        }

        constructor(style: SplitPaneStyle) {
            this.handle = style.handle
        }
    }
}