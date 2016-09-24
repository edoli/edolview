package kr.edoli.imview

import com.badlogic.gdx.Game
import com.badlogic.gdx.Screen

/**
 * Created by daniel on 16. 9. 24.
 */
class BaseApplicationListener(val initScreen: Screen) : Game() {
    override fun create() {
        setScreen(initScreen)
    }
}