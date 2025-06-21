package com.badlogic.gdx.scenes.scene2d.ui

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.Cursor
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.InputListener
import com.badlogic.gdx.scenes.scene2d.utils.Drawable
import com.badlogic.gdx.utils.Align
import com.badlogic.gdx.utils.Null
import kr.edoli.edolview.util.hasFlag

class CustomWindow(title: String, val style: CustomWindowStyle): Window(title, style) {
    val MOVE: Int = 1 shl 5

    init {
        listeners.removeIndex(0)


        addListener(object : InputListener() {
            var startX: Float = 0f
            var startY: Float = 0f
            var lastX: Float = 0f
            var lastY: Float = 0f

            fun updateCursor() {
                if (edge.hasFlag(Align.left or Align.top) || edge.hasFlag(Align.right or Align.bottom)) {
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.NWSEResize)
                } else if (edge.hasFlag(Align.left or Align.bottom) || edge.hasFlag(Align.right or Align.top)) {
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.NESWResize)
                } else if (edge and Align.left != 0 || edge and Align.right != 0) {
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.HorizontalResize)
                } else if (edge and Align.bottom != 0 || edge and Align.top != 0) {
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.VerticalResize)
                } else {
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.Arrow)
                }
            }

            private fun updateEdge(x: Float, y: Float) {
                var border = resizeBorder / 2f
                val width = width
                val height = height
                val padTop = getPadTop()
                val left = 0f
                val right = width
                val bottom = 0f
                edge = 0
                if (isResizable && x >= left - border && x <= right + border && y >= bottom - border) {
                    if (x < left + border) edge = edge or Align.left
                    if (x > right - border) edge = edge or Align.right
                    if (y < bottom + border) edge = edge or Align.bottom
                    if (edge != 0) border += 25f
                    if (x < left + border) edge = edge or Align.left
                    if (x > right - border) edge = edge or Align.right
                    if (y < bottom + border) edge = edge or Align.bottom
                }

                if (isMovable && edge == 0 && y <= height && y >= height - padTop && x >= left && x <= right) {
                    edge = MOVE
                }

                updateCursor()
            }

            override fun touchDown(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int): Boolean {
                if (button == 0) {
                    updateEdge(x, y)
                    dragging = edge != 0
                    startX = x
                    startY = y
                    lastX = x - width
                    lastY = y - height
                }
                return edge != 0 || isModal
            }

            override fun touchUp(event: InputEvent, x: Float, y: Float, pointer: Int, button: Int) {
                dragging = false
            }

            override fun touchDragged(event: InputEvent, x: Float, y: Float, pointer: Int) {
                if (!dragging) return
                var width = width
                var height = height
                var windowX = getX()
                var windowY = getY()

                val minWidth = minWidth
                val maxWidth = maxWidth
                val minHeight = minHeight
                val maxHeight = maxHeight
                val stage = stage
                val clampPosition = keepWithinStage && stage != null && parent === stage.root

                if (stage == null) {
                    return
                }

                if ((edge and MOVE) != 0) {
                    val amountX = x - startX
                    val amountY = y - startY
                    windowX += amountX
                    windowY += amountY
                }
                if ((edge and Align.left) != 0) {
                    var amountX = x - startX
                    if (width - amountX < minWidth) amountX = -(minWidth - width)
                    if (clampPosition && windowX + amountX < 0) amountX = -windowX
                    width -= amountX
                    windowX += amountX
                }
                if ((edge and Align.bottom) != 0) {
                    var amountY = y - startY
                    if (height - amountY < minHeight) amountY = -(minHeight - height)
                    if (clampPosition && windowY + amountY < 0) amountY = -windowY
                    height -= amountY
                    windowY += amountY
                }
                if ((edge and Align.right) != 0) {
                    var amountX = x - lastX - width
                    if (width + amountX < minWidth) amountX = minWidth - width
                    if (clampPosition && windowX + width + amountX > stage.width) amountX =
                        stage.width - windowX - width
                    width += amountX
                }
                if ((edge and Align.top) != 0) {
                    var amountY = y - lastY - height
                    if (height + amountY < minHeight) amountY = minHeight - height
                    if (clampPosition && windowY + height + amountY > stage.height) amountY =
                        stage.height - windowY - height
                    height += amountY
                }
                setBounds(
                    windowX, windowY, width, height,
                )
            }

            override fun mouseMoved(event: InputEvent, x: Float, y: Float): Boolean {
                updateEdge(x, y)
                return isModal
            }

            override fun enter(event: InputEvent?, x: Float, y: Float, pointer: Int, fromActor: Actor?) {
                updateCursor()
                super.enter(event, x, y, pointer, fromActor)
            }

            override fun exit(event: InputEvent?, x: Float, y: Float, pointer: Int, toActor: Actor?) {
                if (!dragging && toActor != this@CustomWindow) {
                    edge = 0
                    Gdx.graphics.setSystemCursor(Cursor.SystemCursor.Arrow)
                }
                super.exit(event, x, y, pointer, toActor)
            }

            fun scrolled(event: InputEvent?, x: Float, y: Float, amount: Int): Boolean {
                return isModal
            }

            override fun keyDown(event: InputEvent, keycode: Int): Boolean {
                return isModal
            }

            override fun keyUp(event: InputEvent, keycode: Int): Boolean {
                return isModal
            }

            override fun keyTyped(event: InputEvent, character: Char): Boolean {
                return isModal
            }
        })
    }

    override fun drawBackground(batch: Batch, parentAlpha: Float, x: Float, y: Float) {
        val borderOverBackground = style.borderOverBackground
        if (edge != 0 && borderOverBackground != null) {
            val color = color
            batch.setColor(color.r, color.g, color.b, color.a * parentAlpha)
            borderOverBackground.draw(batch, x, y, width, height)
        } else {
            if (background != null) {
                val color = color
                batch.setColor(color.r, color.g, color.b, color.a * parentAlpha)
                background.draw(batch, x, y, width, height)
            }
        }

        // Manually draw the title table before clipping is done.
        titleTable.color.a = color.a
        val padTop = getPadTop()
        val padLeft = getPadLeft()
        titleTable.setSize(width - padLeft - getPadRight(), padTop)
        titleTable.setPosition(padLeft, height - padTop)
        drawTitleTable = true
        titleTable.draw(batch, parentAlpha)
        drawTitleTable = false
    }

    class CustomWindowStyle: Window.WindowStyle() {
        @Null
        var borderOverBackground: Drawable? = null
    }
}