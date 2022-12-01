package kr.edoli.edolview.ui.custom

import com.badlogic.gdx.scenes.scene2d.InputEvent
import com.badlogic.gdx.scenes.scene2d.ui.Button
import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.utils.ClickListener

class DropDownMenu<T>(
        button: Button, listStyle: List.ListStyle, onSelectItem: ((T) -> Unit)? = null
) : DropDown(button, List<T>(listStyle)) {
    @Suppress("UNCHECKED_CAST")
    val list: List<T> = dropDown as List<T>

    init {
        list.addListener(object : ClickListener() {
            override fun clicked(event: InputEvent, x: Float, y: Float) {
                val item = list.selected
                list.selection.clear()
                if (onSelectItem != null) {
                    onSelectItem(item)
                }

                hideDropDown()
            }
        })
    }

    override fun showDropDown() {
        if (list.items.size > 0) {
            super.showDropDown()
        }
    }
}

