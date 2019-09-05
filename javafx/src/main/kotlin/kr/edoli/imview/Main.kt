package kr.edoli.imview

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration
import kr.edoli.imview.ui.App

fun main() {
    val config = Lwjgl3ApplicationConfiguration()
    config.useOpenGL3(true, 3, 2)
    Lwjgl3Application(App(), config)

    ImContext.mainFile.subscribe { Gdx.graphics.setTitle(it.name) }
}