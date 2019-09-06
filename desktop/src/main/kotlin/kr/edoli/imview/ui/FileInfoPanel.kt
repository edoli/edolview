package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Label
import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext

class FileInfoPanel : Panel(false) {
    init {
        val bitPerPixelLabel = Label("", UIFactory.skin).tooltip("Bits per pixel")
        val maxValueLabel = Label("", UIFactory.skin).tooltip("Max value for type")
        val numChannelsLabel = Label("", UIFactory.skin).tooltip("Number of channels")

        add("Bits")
        add("Max")
        add("Channels")

        row()

        add(bitPerPixelLabel)
        add(maxValueLabel)
        add(numChannelsLabel)

        ImContext.mainImageSpec.subscribe { imageSpec ->
            if (imageSpec == null) return@subscribe

            bitPerPixelLabel.setText("${imageSpec.bitsPerPixel}")
            maxValueLabel.setText("${imageSpec.maxValue}")
            numChannelsLabel.setText("${imageSpec.numChannels}")
        }
    }
}