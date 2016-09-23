package kr.edoli.imview.ui.view

import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edoliui.res.FontAwesomes
import kr.edoli.edoliui.widget.drawable.ColorDrawable
import kr.edoli.imview.bus.Bus
import kr.edoli.imview.bus.SelectionCopyMessage
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.UI
import kr.edoli.imview.ui.onClick

/**
 * Created by daniel on 16. 9. 23.
 */
class ContextMenu(val contextGroup: ContextGroup) : Table() {

    init {
        background = ColorDrawable(Colors.background)

        addButton(FontAwesomes.FaCopy) {
            Bus.send(SelectionCopyMessage())
        }

        addButton(FontAwesomes.FaImage) {

        }

        addButton(FontAwesomes.FaFilter) {

        }

        addButton(FontAwesomes.FaBarChart) {

        }

        row()

        addButton(FontAwesomes.FaCopy) {

        }

        addButton(FontAwesomes.FaCopy) {

        }

        addButton(FontAwesomes.FaCopy) {

        }

        addButton(FontAwesomes.FaCopy) {

        }

        pack()
    }

    override fun draw(batch: Batch, parentAlpha: Float) {
        super.draw(batch, parentAlpha)
    }

    fun addButton(icon: String , handle: (Actor) -> Unit) {
        val button = UI.iconButton(icon)
        button.onClick(handle)
        button.onClick { contextGroup.hide() }
        add(button).size(32f).pad(2f)
    }
}