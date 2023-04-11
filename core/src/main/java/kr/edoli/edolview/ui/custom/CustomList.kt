package kr.edoli.edolview.ui.custom

import com.badlogic.gdx.scenes.scene2d.ui.List
import com.badlogic.gdx.scenes.scene2d.ui.Skin

class CustomList<T>(style: ListStyle, val textFunc: (T) -> String) : List<T>(style) {

    constructor(skin: Skin, textFunc: (T) -> String) : this(skin[ListStyle::class.java], textFunc)
    override fun toString(item: T): String {
        return textFunc(item)
    }
}