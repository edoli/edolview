package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.custom.Formatters
import kr.edoli.imview.ui.custom.NumberLabel

class FileInfoPanel : Panel(false) {
    init {
        val fileNameLabel = UIFactory.createLabel(ImContext.mainFileName)
        val fileDirLabel = UIFactory.createLabel(ImContext.mainFileDirectory)
        val bitPerPixelLabel = NumberLabel("Bits per pixel", UIFactory.skin, Formatters.intFormatter)
        val maxValueLabel = NumberLabel("Max value for type", UIFactory.skin, Formatters.intFormatter)
        val numChannelsLabel = NumberLabel("Number of channels", UIFactory.skin, Formatters.intFormatter)

        add(fileNameLabel)
        row()
        add(fileDirLabel)
        row()

        add(Table(skin).apply {
            add("Bits").spaceRight(8f)
            add("Max").spaceRight(8f)
            add("Channels")

            row()

            add(bitPerPixelLabel)
            add(maxValueLabel)
            add(numChannelsLabel)
        })

        ImContext.mainImageSpec.subscribe { imageSpec ->
            if (imageSpec == null) return@subscribe

            bitPerPixelLabel.value = imageSpec.bitsPerPixel.toDouble()
            maxValueLabel.value = imageSpec.maxValue
            numChannelsLabel.value = imageSpec.numChannels.toDouble()
        }
    }
}