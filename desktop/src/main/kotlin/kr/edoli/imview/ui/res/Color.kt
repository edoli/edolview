package kr.edoli.imview.ui.res

import com.badlogic.gdx.graphics.Color
import kotlin.reflect.full.declaredMemberProperties


object Colors {
    val normal = Color.WHITE
    val negative = Color.RED
    val over = Color.LIGHT_GRAY
    val inactive = Color.LIGHT_GRAY
    val accent = Color.GREEN

    init {
        Colors::class.declaredMemberProperties.forEach {
            com.badlogic.gdx.graphics.Colors.put(it.name, it.get(this) as Color)
        }
    }
}