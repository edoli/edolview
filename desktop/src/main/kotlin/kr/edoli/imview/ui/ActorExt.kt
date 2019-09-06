package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.ui.TextTooltip

fun <T: Actor> T.tooltip(text: String): T {
    addListener(TextTooltip(text, UIFactory.tooltipManager, UIFactory.skin))
    return this
}