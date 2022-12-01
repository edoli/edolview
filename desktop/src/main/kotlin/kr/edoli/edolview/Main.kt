package kr.edoli.edolview

import com.badlogic.gdx.backends.lwjgl3.Lwjgl3ApplicationConfiguration
import com.badlogic.gdx.backends.lwjgl3.Lwjgl3WindowAdapter
import com.badlogic.gdx.backends.lwjgl3.MyLwjgl3Application
import com.badlogic.gdx.graphics.glutils.HdpiMode
import io.sentry.Sentry
import kotlinx.cli.ArgParser
import kotlinx.cli.ArgType
import kotlinx.cli.default
import kr.edoli.edolview.ui.App
import javax.swing.UIManager
import kotlin.system.exitProcess


fun main(args: Array<String>) {
    Sentry.init { options ->
        options.dsn = "https://f081c5cc03134cf885a80d19006763bb@o191004.ingest.sentry.io/6524268"
        options.tracesSampleRate = 1.0
    }

    try {
        val parser = ArgParser("edolview")
        val initPath by parser.option(ArgType.String, shortName = "i", description = "Input path")
        val devMode by parser.option(ArgType.Boolean, shortName = "d", description = "Turn on dev mode").default(false)
        parser.parse(args)

        UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName())

        val config = Lwjgl3ApplicationConfiguration()
        config.setOpenGLEmulation(Lwjgl3ApplicationConfiguration.GLEmulation.GL30, 3, 2)
        config.setHdpiMode(HdpiMode.Logical)
        config.setWindowListener(object : Lwjgl3WindowAdapter() {
            override fun filesDropped(files: Array<String>) {
                ImContext.mainPath.update(files[0])
            }

            override fun closeRequested(): Boolean {
                exitProcess(0)
            }
        })
        config.setTitle("Edolview")
        config.setWindowedMode(1280, 720)
        config.setWindowIcon("icon.png")

        MyLwjgl3Application(App(initPath), config)
    } catch (e: Exception) {
        Sentry.captureException(e)
    }
}