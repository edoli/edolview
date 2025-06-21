package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.scenes.scene2d.ui.Table
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.channelNames
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import org.opencv.core.Core
import org.opencv.core.Mat

class PlotPanel : Panel(false) {

    private var reduceDim = 0
    val plotWidget = PlotWidget()
    val buttons = Table()

    init {
        add(plotWidget).expandX().fillX().height(128f)
        row()
        add(buttons).expandX().fillX()
        row()
        add(UIFactory.createTextButton("Vertical") {
            reduceDim = if (it.isChecked) 1 else 0
            updatePlot()
        }.apply {
            isChecked = false
            style = UIFactory.textToggleButtonStyle
        }).expandX().fillX()

        ImContext.marqueeImage.subscribe(this, "Plot panel") { mat ->
            if (!isGone()) {
                updatePlot()
            } else {
                plotWidget.setData(null)
            }
        }

        onGoneChanged = {
            if (!it) {
                updatePlot()
            } else {
                plotWidget.setData(null)
            }
        }
    }

    private fun updatePlot() {
        buttons.clearChildren()

        val mat = ImContext.marqueeImage.get()
        if (mat != null) {
            val reducedMat = if (reduceDim == 0) Mat(1, mat.cols(), mat.type()) else
                Mat(mat.rows(), 1, mat.type())
            Core.reduce(mat, reducedMat, reduceDim, Core.REDUCE_AVG)

            val marqueeBox = ImContext.marqueeBox.get()
            val xAxis = if (reduceDim == 0) PlotIntRange(marqueeBox.x..marqueeBox.x+marqueeBox.width) else PlotIntRange(marqueeBox.y..marqueeBox.y+marqueeBox.height)

            plotWidget.xAxisName = if (reduceDim == 0) "X" else "Y"
            plotWidget.xAxis = xAxis
            plotWidget.setData(reducedMat)

            reducedMat.release()

            val names = mat.channelNames()

            (names.indices).forEach { index ->
                buttons.add(UIFactory.createTextButton(names[index]) {
                    plotWidget.isShow[index] = it.isChecked
                }.apply {
                    isChecked = plotWidget.isShow[index]
                    style = UIFactory.textToggleButtonStyle
                })
            }
        } else {
            plotWidget.setData(null)
        }
    }
}