package kr.edoli.imview.ui

import kr.edoli.imview.ImContext

class FileInfoPanel : Panel(false) {
    init {
        val bitPerPixelLabel = NumberLabel("Bits per pixel", UIFactory.skin, Formatters.intFormatter)
        val maxValueLabel = NumberLabel("Max value for type", UIFactory.skin, Formatters.intFormatter)
        val numChannelsLabel = NumberLabel("Number of channels", UIFactory.skin, Formatters.intFormatter)

        add("Bits").spaceRight(8f)
        add("Max").spaceRight(8f)
        add("Channels")

        row()

        add(bitPerPixelLabel)
        add(maxValueLabel)
        add(numChannelsLabel)

        ImContext.mainImageSpec.subscribe { imageSpec ->
            if (imageSpec == null) return@subscribe

            bitPerPixelLabel.value = imageSpec.bitsPerPixel.toDouble()
            maxValueLabel.value = imageSpec.maxValue
            numChannelsLabel.value = imageSpec.numChannels.toDouble()
        }
    }
}