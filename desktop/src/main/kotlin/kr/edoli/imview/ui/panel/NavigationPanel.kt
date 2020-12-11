package kr.edoli.imview.ui.panel

import com.badlogic.gdx.Gdx
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory
import kr.edoli.imview.ui.res.Ionicons

class NavigationPanel : Panel(false) {
    init {
        add(UIFactory.createSlider(Ionicons.ionMdPlay, 0.0f, Gdx.graphics.displayMode.refreshRate.toFloat(),
                0.01f, ImContext.frameSpeed)).expandX().fillX()
    }
}