package kr.edoli.imview.ui

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.Pixmap
import com.badlogic.gdx.graphics.Texture

/**
 * Created by daniel on 16. 9. 10.
 */

val White = colorTexture(Color.WHITE)
val RED = colorTexture(Color.RED)

fun colorTexture(color: Color) : Texture {
    println(color)

    val pixmap = Pixmap(1, 1, Pixmap.Format.RGBA8888)

    pixmap.setColor(color)
    pixmap.fill()

    val texture = Texture(pixmap)

    pixmap.dispose()

    return texture
}