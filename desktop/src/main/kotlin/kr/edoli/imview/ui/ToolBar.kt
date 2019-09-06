package kr.edoli.imview.ui

import com.badlogic.gdx.scenes.scene2d.utils.NinePatchDrawable
import com.badlogic.gdx.utils.Align
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.ClipboardUtils
import kr.edoli.imview.image.MarqueeUtils

class ToolBar : Panel() {

    companion object {
        const val barHeight = 24f
        const val iconWidth = 24f
    }

    init {
        background = NinePatchDrawable(UIFactory.skin.atlas.createPatch("default-pane"))

        align(Align.left)

        add(UIFactory.createIconButton(Ionicons.ionMdSave) {
            MarqueeUtils.saveImage(false)
        }.tooltip("Save selected image")).width(iconWidth)

        add(UIFactory.createIconButton(Ionicons.ionMdClipboard) {
            ClipboardUtils.showClipboardImage()
        }.tooltip("Show clipboard")).width(iconWidth)

        add().width(32f)

        add(UIFactory.createIconButton(Ionicons.ionMdExpand) {
            ImContext.fitImage.onNext(true)
        }.tooltip("Fit selection to view")).width(iconWidth)
        add(UIFactory.createIconButton(Ionicons.ionMdContract) {
            ImContext.centerImage.onNext(true)
        }.tooltip("Center selection view")).width(iconWidth)

        add().width(32f)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdPaper, ImContext.isShowInfo).tooltip("Show information")).width(iconWidth)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdOptions, ImContext.isShowController).tooltip("Show controller")).width(iconWidth)
        add(UIFactory.createToggleIconButton(Ionicons.ionMdAdd, ImContext.isShowCrosshair).tooltip("Show crosshair")).width(iconWidth)
    }
}