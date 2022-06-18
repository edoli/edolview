package kr.edoli.imview.util

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.backends.lwjgl3.MyLwjgl3Application
import org.lwjgl.glfw.GLFW
import java.awt.GraphicsEnvironment

class OS {
    companion object {
        fun getScalingFactor(): Float {
            val ge = GraphicsEnvironment.getLocalGraphicsEnvironment().defaultScreenDevice.defaultConfiguration
            val transform = ge.defaultTransform
            return transform.scaleX.toFloat()
        }
    }
}