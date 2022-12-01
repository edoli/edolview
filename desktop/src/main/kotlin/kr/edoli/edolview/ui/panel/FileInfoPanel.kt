package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.ui.custom.Formatters
import kr.edoli.edolview.ui.custom.NumberLabel
import kr.edoli.edolview.ui.res.Ionicons
import java.awt.Desktop
import java.io.File

class FileInfoPanel : Panel(false) {
    init {
        val fileNameLabel = UIFactory.createLabel(ImContext.mainFileName)
        val fileDirLabel = UIFactory.createLabel(ImContext.mainFileDirectory)
        val bitPerPixelLabel = NumberLabel("Bits per pixel", skin, Formatters.intFormatter)
        val maxValueLabel = NumberLabel("Max value for type", skin, Formatters.intFormatter)
        val numChannelsLabel = NumberLabel("Number of channels", skin, Formatters.intFormatter)

        add(fileNameLabel)
        row()
        add(fileDirLabel)
        row()
        add(UIFactory.createIconButton(Ionicons.ionMdFolder) {
            val file = File(ImContext.mainFileDirectory.get())
            if (file.isDirectory) {
                Desktop.getDesktop().open(file)
            }
        })
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

        ImContext.mainImageSpec.subscribe(this, "Update file info") { imageSpec ->
            if (imageSpec == null) return@subscribe

            bitPerPixelLabel.value = imageSpec.bitsPerPixel.toDouble()
            maxValueLabel.value = imageSpec.typeMaxValue
            numChannelsLabel.value = imageSpec.numChannels.toDouble()
        }
    }
}