package kr.edoli.imview

import com.badlogic.gdx.Game
import com.badlogic.gdx.Gdx
import kr.edoli.imview.ui.screen.MainScreen

/**
 * Created by daniel on 16. 9. 10.
 */

class ImView : Game() {

    override fun create() {
        setScreen(MainScreen())


        val runnable = object : Runnable {

            private var lastTime: Long = 0

            override fun run() {
                var currentTime = System.nanoTime()

                while (currentTime - lastTime < 16000000) {
                    try {
                        Thread.sleep(16)
                    } catch (e: InterruptedException) {
                        e.printStackTrace()
                    }
                    currentTime = System.nanoTime()
                }
                Gdx.app.postRunnable(this)

                lastTime = currentTime
            }
        }

        Gdx.app.postRunnable(runnable)
    }
}