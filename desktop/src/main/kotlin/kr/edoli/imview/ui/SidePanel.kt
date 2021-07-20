package kr.edoli.imview.ui

import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.drawable.BorderedDrawable
import kr.edoli.imview.ui.panel.CollapsiblePanel
import kr.edoli.imview.ui.panel.DisplayProfilePanel
import kr.edoli.imview.ui.panel.NavigationPanel
import kr.edoli.imview.ui.panel.StatisticsPanel
import kr.edoli.imview.ui.panel.histogram.HistogramPanel

class SidePanel : Panel() {
    init {
        (background as BorderedDrawable).apply {
            isBorder = false
        }

        align(Align.top)

        add(CollapsiblePanel("View controller", DisplayProfilePanel(), false)).fillX()

        addHorizontalDivider().padTop(4f).padBottom(4f)

        add(CollapsiblePanel("Image Navigator", NavigationPanel(), true)).fillX()

        addHorizontalDivider().padTop(4f).padBottom(4f)

//        add(CollapsiblePanel("Image statistics", StatisticsPanel(ImContext.mainImage), true)).fillX()

        addHorizontalDivider().padTop(4f).padBottom(4f)

        // add(CollapsiblePanel("Selection statistics", StatisticsPanel(ImContext.marqueeImage), true)).fillX()

        // addHorizontalDivider().padTop(4f).padBottom(4f)

        // Temporary disable histogram
        val histogramPanel = HistogramPanel()
        val histogramCollapsible = CollapsiblePanel("Histogram", histogramPanel, true)
        ImContext.mainImage.subscribe(this, "Update histogram") {
            if (!histogramCollapsible.collapse) {
                histogramPanel.updateHistogram()
            }
        }

        histogramCollapsible.onCollapseChanged = { isCollapse ->
            if (!isCollapse) {
                histogramPanel.updateHistogram()
            }
        }
        add(histogramCollapsible).fillX()

        ImContext.isShowController.subscribe(this, "Layout") {
            isVisible = it
        }
    }
}