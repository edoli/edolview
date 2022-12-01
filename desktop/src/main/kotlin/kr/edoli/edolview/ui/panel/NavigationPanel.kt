package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.ui.res.Ionicons
import kr.edoli.edolview.ui.res.uiSkin

class NavigationPanel : Panel(false) {
    init {
        add(Table().apply {
            add(Label("Interval", uiSkin))
            add().width(4f)
            add(UIFactory.createIntField(ImContext.frameInterval)).width(100f)
        }).expandX().fillX()
        row()
        add(UIFactory.createSlider(Ionicons.ionMdPlay, 0.0f, Gdx.graphics.displayMode.refreshRate.toFloat(),
                0.01f, ImContext.frameSpeed)).expandX().fillX()
    }
}