package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.asset.Asset
import kr.edoli.edolview.asset.ListAsset
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory

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

        onGoneChanged = {
            if (!it) {
                ImContext.mainAsset.update(ImContext.listAsset)
            } else {
                ImContext.mainAsset.update(null)
            }
        }
    }
}