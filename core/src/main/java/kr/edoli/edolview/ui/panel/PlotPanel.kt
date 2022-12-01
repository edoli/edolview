package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.graphics.Color
import com.badlogic.gdx.graphics.g2d.Batch
import com.badlogic.gdx.graphics.glutils.ShapeRenderer
import com.badlogic.gdx.scenes.scene2d.ui.Widget
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.toDoubleArray
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import org.opencv.core.Core
import org.opencv.core.Mat

class PlotPanel : Panel(false) {

    private var reduceDim = 0
    val plotWidget = PlotWidget()

    init {
        add(plotWidget).expandX().fillX().height(128f)
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
                plotWidget.data = null
            }
        }

        onGoneChanged = {
            if (!it) {
                updatePlot()
            } else {
                plotWidget.data = null
            }
        }
    }

    private fun updatePlot() {
        val mat = ImContext.marqueeImage.get()
        if (mat != null) {
            val reducedMat = if (reduceDim == 0) Mat(1, mat.cols(), mat.type()) else
                Mat(mat.rows(), 1, mat.type())
            Core.reduce(mat, reducedMat, reduceDim, Core.REDUCE_AVG)
            val data = reducedMat.toDoubleArray()

            reducedMat.release()

            plotWidget.channels = reducedMat.channels()
            plotWidget.data = FloatArray(data.size) { data[it].toFloat() }
        } else {
            plotWidget.data = null
        }
    }

    class PlotWidget : Widget() {
        val shapeRenderer = ShapeRenderer()
        var data: FloatArray? = null
        var channels = 0


        override fun draw(batch: Batch, parentAlpha: Float) {
            super.draw(batch, parentAlpha)

            val data = this.data

            if (data != null) {
                batch.end()

                shapeRenderer.projectionMatrix = batch.projectionMatrix
                shapeRenderer.transformMatrix = batch.transformMatrix
                shapeRenderer.begin(ShapeRenderer.ShapeType.Line)

                val maxValue = data.maxOrNull() ?: return
                var minValue = data.minOrNull() ?: return
                if (minValue == maxValue) {
                    minValue = maxValue - 1.0f
                }

                val scale = height / (maxValue - minValue)
                val size = data.size / channels
                val step = width / (size - 1)

                val colors = if (channels == 1) {
                    arrayOf(Color.GRAY)
                } else {
                    arrayOf(
                            Color.RED,
                            Color.GREEN,
                            Color.BLUE,
                            Color.GRAY
                    )
                }

                for (c in 0 until channels) {
                    shapeRenderer.color = colors[c]

                    var offsetX = x
                    var beforeY = (data[c] - minValue) * scale + y
                    for (i in 1 until size) {
                        val curY = (data[c + i * channels] - minValue) * scale + y
                        shapeRenderer.line(offsetX, beforeY, offsetX + step, curY)
                        offsetX += step
                        beforeY = curY
                    }
                }
                shapeRenderer.end()

                batch.begin()
            }
        }
    }
}