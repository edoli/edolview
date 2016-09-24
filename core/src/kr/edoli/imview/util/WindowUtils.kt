package kr.edoli.imview.util

import com.badlogic.gdx.ApplicationListener
import com.badlogic.gdx.Gdx
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Application
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3Window
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowConfiguration
import java.util.*

/**
 * Created by daniel on 16. 9. 24.
 */
object WindowUtils {

    private val windowMap = HashMap<Windows, Long>()

    fun getWindows(): com.badlogic.gdx.utils.Array<Lwjgl3Window> {
        val app = Gdx.app as Lwjgl3Application
        val claz = Lwjgl3Application::class.java
        val field = claz.getDeclaredField("windows")
        field.isAccessible = true
        return field.get(app) as com.badlogic.gdx.utils.Array<Lwjgl3Window>
    }

    fun getWindow(index: Int): Lwjgl3Window {
        val windows = getWindows()
        return windows[index]
    }

    fun currentWindow(): Lwjgl3Window {
        val app = Gdx.app as Lwjgl3Application
        val claz = Lwjgl3Application::class.java
        val field = claz.getDeclaredField("currentWindow")
        field.isAccessible = true

        return field.get(app) as Lwjgl3Window
    }

    fun getHandle(window: Lwjgl3Window): Long {
        val claz = Lwjgl3Window::class.java
        val field = claz.getDeclaredField("windowHandle")
        field.isAccessible = true

        return field.get(window) as Long
    }

    fun hasWindow(windowName: Windows): Boolean {
        return windowMap.containsKey(windowName)
    }

    fun closeWindow(index: Int) {
        getWindow(index).closeWindow()
    }

    fun removeWindow(windowName: Windows) {
        windowMap.remove(windowName)
    }

    fun getIndexOf(windowName: Windows): Int {
        val windows = getWindows()
        val handle = windowMap[windowName]
        for ((i, window) in windows.withIndex()) {
            if (getHandle(window) == handle) {
                return i
            }
        }
        return -1
    }

    fun newWindow(windowName: Windows, listener: ApplicationListener, config: Lwjgl3WindowConfiguration): Lwjgl3Window {
        val app = Gdx.app as Lwjgl3Application
        val window = app.newWindow(listener, config)
        windowMap[windowName] = getHandle(window)
        return window
    }
}