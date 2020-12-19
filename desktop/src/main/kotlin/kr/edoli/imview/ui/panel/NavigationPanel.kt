package kr.edoli.imview.ui.panel

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory
import kr.edoli.imview.ui.res.Ionicons
import kr.edoli.imview.ui.res.uiSkin

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