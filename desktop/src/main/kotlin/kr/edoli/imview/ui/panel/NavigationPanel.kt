package kr.edoli.imview.ui.panel

import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.Panel
import kr.edoli.imview.ui.UIFactory
import kr.edoli.imview.ui.res.Ionicons

class NavigationPanel() : Panel(false) {
    init {

        add(UIFactory.createSlider(Ionicons.ionMdPlay, 0.0f, 1.0f, 0.01f, ImContext.frameSpeed)).expandX().fillX()
    }
}