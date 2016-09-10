package kr.edoli.imview

import com.badlogic.gdx.Game
import kr.edoli.imview.ui.screen.MainScreen

/**
 * Created by daniel on 16. 9. 10.
 */

class ImView : Game() {
    override fun create() {
        setScreen(MainScreen())
    }
}