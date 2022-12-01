package kr.edoli.edolview.ui

import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.ui.drawable.BorderedDrawable
import kr.edoli.edolview.ui.panel.*
import kr.edoli.edolview.ui.panel.histogram.HistogramPanel

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

//        addHorizontalDivider().padTop(4f).padBottom(4f)

         add(CollapsiblePanel("Selection statistics", StatisticsPanel(ImContext.marqueeImage), true)).fillX()

         addHorizontalDivider().padTop(4f).padBottom(4f)

        add(CollapsiblePanel("Histogram", HistogramPanel(), true)).fillX()

        addHorizontalDivider().padTop(4f).padBottom(4f)

        add(CollapsiblePanel("Plot", PlotPanel(), true)).fillX()

        addHorizontalDivider().padTop(4f).padBottom(4f)

        ImContext.isShowController.subscribe(this, "Layout") {
            isVisible = it
        }
    }
}