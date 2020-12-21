package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.image.MarqueeUtils
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.ui.res.Ionicons
import kr.edoli.imview.ui.window.ShaderEditor

class ToolBar : Panel() {

    companion object {
        const val barHeight = 24f
        const val iconWidth = 24f
    }

    init {
        align(Align.left)

        add(UIFactory.createIconButton(Ionicons.ionMdSync) {
            ImageStore.clearCache()
        }.tooltip("Clear cache")).width(iconWidth)

        add(UIFactory.createIconButton(Ionicons.ionMdSave) {
            MarqueeUtils.saveImage(false)
        }.tooltip("Save selected image")).width(iconWidth)

        add(UIFactory.createIconButton(Ionicons.ionMdImage) {
            ClipboardUtils.showClipboardImage()
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
            ShaderEditor()
        }).width(iconWidth)

        add().expandX()
        add(UIFactory.createToggleIconButton(Ionicons.ionMdOptions, ImContext.isShowController)).width(iconWidth)
    }
}