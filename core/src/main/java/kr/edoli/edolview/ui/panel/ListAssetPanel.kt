package kr.edoli.edolview.ui.panel

import com.badlogic.gdx.utils.Align
import kr.edoli.edolview.ImContext
import kr.edoli.edolview.asset.Asset
import kr.edoli.edolview.ui.Panel
import kr.edoli.edolview.ui.UIFactory
import kr.edoli.edolview.util.ObservableList
import kotlin.math.min

class ListAssetPanel(observableList: ObservableList<Asset>) : Panel(false) {

    init {
        add(UIFactory.createList(observableList, Asset::name) {
            ImContext.mainAsset.update(it)
        }.apply {
            alignment = Align.right

            onRemoveItemIndex = { index ->
                val list = observableList.items
                val newList = list.toMutableList()
                newList.removeAt(index)
                observableList.update(newList, min(newList.size - 1, index))
            }
        }).minWidth(0f).expandX().fillX()
    }
}