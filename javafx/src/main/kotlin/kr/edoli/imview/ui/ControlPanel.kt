package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Table
import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import kr.edoli.imview.ImContext

class ControlPanel : Table() {
    init {
        background = NinePatchDrawable(UIFactory.skin.atlas.createPatch("default-pane"))
        add(UIFactory.createToggleTextButton("Smoothing", ImContext.smoothing))
        row()
        add(UIFactory.createToggleTextButton("Normalize", ImContext.normalize))
        row()
        add(UIFactory.createSlider(-10f, 10f, 0.1f, ImContext.imageBrightness))
        row()
        add(UIFactory.createSlider(-10f, 10f, 0.1f, ImContext.imageContrast))
        row()
        add(UIFactory.createSlider(0f, 10f, 0.1f, ImContext.imageGamma))
    }
}