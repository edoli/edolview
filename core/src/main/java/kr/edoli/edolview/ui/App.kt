package kr.edoli.edolview.ui

import com.badlogic.gdx.Game
import com.badlogic.gdx.Gdx
import com.badlogic.gdx.graphics.glutils.ShaderProgram
import com.badlogic.gdx.utils.Timer
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.PeriodicTask
import kr.edoli.edolview.asset.Asset
import kr.edoli.edolview.net.ImageServer
import org.opencv.core.Core

class App(private val initPath: String?) : Game() {
    companion object {
        init {
            System.loadLibrary(Core.NATIVE_LIBRARY_NAME.replace("450", "4"))
        }
    }

    override fun create() {
        ShaderProgram.pedantic = false
        if (initPath != null) {
            ImContext.mainAsset.update(Asset.fromUri(initPath))
        }
        ImContext.mainTitle.subscribe(this, "Update title") { Gdx.graphics.setTitle(it) }
        Gdx.graphics.isContinuousRendering = false

        setScreen(MainScreen())

        Timer.schedule(PeriodicTask, 1.0f, 0.2f)

        ImageServer.create()
    }
}