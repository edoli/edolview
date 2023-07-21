package kr.edoli.edolview.ui

import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.asset.Asset
import kr.edoli.edolview.store.ImageStore
import kr.edoli.edolview.ui.drawable.BorderedDrawable
import kr.edoli.edolview.ui.res.Colors
import kr.edoli.edolview.ui.res.Ionicons
import kr.edoli.edolview.ui.window.ShaderEditor

class ToolBar : Panel() {

    companion object {
        const val barHeight = 24f
        const val iconWidth = 24f
    }

    init {
        background = BorderedDrawable(Colors.background, Colors.backgroundBorder).apply {
            topBorder = false
            leftBorder = false
            rightBorder = false
        }

        align(Align.left)

        add(UIFactory.createDropdownMenu("Recent files ▼", ImContext.recentAssets, Asset::name) { asset ->
            ImContext.mainAsset.update(asset)
        })

        add().width(16f)

        add(UIFactory.createIconButton(Ionicons.ionMdTrash) {
            ImageStore.clearCache()
        }.tooltip("Clear cache")).width(iconWidth)

        add(UIFactory.createToggleIconButton(Ionicons.ionMdSync, ImContext.autoRefresh)).width(iconWidth)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdCalendar, ImContext.isShowBackground)).width(iconWidth)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdApps, ImContext.isShowPixelValue)).width(iconWidth)

        add(UIFactory.createIconButton(Ionicons.ionMdClipboard) {
            ImContext.loadFromClipboard()
        }.tooltip("Show clipboard")).width(iconWidth)

        add(UIFactory.createToggleIconButton(Ionicons.ionMdInformationCircleOutline, ImContext.isShowFileInfo)).width(iconWidth)

        add().width(32f)

        add(UIFactory.createIconButton(Ionicons.ionMdExpand) {
            ImContext.fitSelection.onNext(true)
        }.tooltip("Fit selection to view")).width(iconWidth)
        add(UIFactory.createIconButton(Ionicons.ionMdContract) {
            ImContext.centerSelection.onNext(true)
        }.tooltip("Center selection view")).width(iconWidth)

        add().width(32f)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdAdd, ImContext.isShowCrosshair)).width(iconWidth)
        add(UIFactory.createIconButton(Ionicons.ionMdPaper) {
            ShaderEditor.show()
        }.tooltip("Edit shader script")).width(iconWidth)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdGitNetwork, ImContext.isServerActive)).width(iconWidth)

        add(UIFactory.createLabel(ImContext.imageServerAddress))

        add().expandX()
        add(UIFactory.createToggleIconButton(Ionicons.ionMdOptions, ImContext.isShowController)).width(iconWidth)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdTv, ImContext.isShowStatusBar)).width(iconWidth)
    }
}