package kr.edoli.edolview.ui

import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.drawable.BorderedDrawable
import kr.edoli.edolview.ui.panel.*
import kr.edoli.edolview.ui.panel.histogram.HistogramPanel

class SidePanel : Panel() {

    private var numPanel = 0

    init {
        (background as BorderedDrawable).apply {
            isBorder = false
        }

        align(Align.top)

        addPanel("View controller", DisplayProfilePanel(), false)
        addPanel("Image Navigator", NavigationPanel(), true)
        addPanel("Selection statistics", StatisticsPanel(ImContext.marqueeImage), true)
        addPanel("Histogram", HistogramPanel(), true)
        addPanel("Plot", PlotPanel(), true)
        addPanel("List assets", ListAssetPanel(ImContext.recentAssets), false)

        ImContext.isShowController.subscribe(this, "Layout") {
            isVisible = it
        }
    }

    fun addPanel(name: String, panel: Panel, initCollapsible: Boolean) {
        if (numPanel == 0) {
            add(CollapsiblePanel(name, panel, initCollapsible)).fillX()
        } else {
            addHorizontalDivider().padTop(4f).padBottom(4f)
            add(CollapsiblePanel(name, panel, initCollapsible)).fillX()
        }

        numPanel += 1
    }
}