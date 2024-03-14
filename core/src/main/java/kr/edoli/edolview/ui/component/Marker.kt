package kr.edoli.edolview.ui.component

import com.badlogic.gdx.scenes.scene2d.Group

abstract class Marker: Group() {

    var imageX: Float = 0.0f
    var imageY: Float = 0.0f

    abstract fun update()
}