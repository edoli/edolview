package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.Touchable
import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable

open class Panel : Table() {
    init {
        background = NinePatchDrawable(UIFactory.skin.atlas.createPatch("default-pane"))
        touchable = Touchable.enabled
    }
}