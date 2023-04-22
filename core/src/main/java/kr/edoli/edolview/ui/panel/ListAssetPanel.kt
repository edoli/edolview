package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.scenes.scene2d.Actor
import com.badlogic.gdx.scenes.scene2d.utils.ChangeListener
import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.minMax
import kr.edoli.edolview.image.pow
import kr.edoli.edolview.image.split
import kr.edoli.edolview.image.sum
import kr.edoli.edolview.res.Asset
import kr.edoli.edolview.res.ListAsset
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.ui.custom.NumberLabel
import kr.edoli.edolview.util.ObservableList
import kr.edoli.edolview.util.forever
import org.opencv.core.Mat
import java.util.concurrent.LinkedBlockingQueue
import kotlin.concurrent.thread
import kotlin.math.sqrt

class ListAssetPanel : Panel(false) {

    init {
        add(UIFactory.createList(ImContext.listAsset.assets, Asset::name) {
            ImContext.mainAsset.update { it }
        }.apply {
            alignment = Align.right
        }).minWidth(0f).expandX().fillX()

        ImContext.mainAsset.subscribe(this, "ListAssetPanel") { asset ->
            isVisible = asset is ListAsset
        }
    }
}