package kr.edoli.imview.ui

import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext
import kr.edoli.imview.ui.panel.DisplayProfilePanel
import kr.edoli.imview.ui.panel.NavigationPanel
import kr.edoli.imview.ui.panel.StatisticsPanel
import kr.edoli.imview.ui.panel.histogram.HistogramPanel

class SidePanel : Panel() {
    init {
        align(Align.top)

        add(CollapsiblePanel("View controller", DisplayProfilePanel(), false)).fillX()

        addHorizontalDivider().pad(4f)

        add(CollapsiblePanel("Image Navigator", NavigationPanel(), true)).fillX()

        addHorizontalDivider().pad(4f)

        add(CollapsiblePanel("Image statistics", StatisticsPanel(ImContext.mainImage), true)).fillX()

        addHorizontalDivider().pad(4f)

        add(CollapsiblePanel("Selection statistics", StatisticsPanel(ImContext.marqueeImage), true)).fillX()

        addHorizontalDivider().pad(4f)

        add(CollapsiblePanel("Histogram", HistogramPanel(), true)).fillX()

        ImContext.isShowController.subscribe {
            isVisible = it
        }
    }
}