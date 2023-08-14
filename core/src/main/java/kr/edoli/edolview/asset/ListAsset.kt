package kr.edoli.edolview.asset

import kr.edoli.edolview.ImContext
import kr.edoli.edolview.image.ImageSpec
import kr.edoli.edolview.util.ObservableList

/**
 * For navigating through multiple assets
 */
class ListAsset : Asset() {
    val assets = ObservableList<Asset>(ArrayList())

    init {
        reset()
    }

    override val name: String
        get() = assets.get()?.name ?: ""

    override val shouldAddToRecentAssets = false


    override fun loadImageSpec(): ImageSpec? {
        return assets.get()?.retrieveImageSpec()
    }

    override fun next(): Asset? {
        sibiling(1)
        return this
    }

    override fun prev(): Asset? {
        sibiling(-1)
        return this
    }

    override fun checkRefresh() = false

    fun reset() {
        val mainAsset = ImContext.mainAsset.get()

        if (mainAsset != null && mainAsset !is ListAsset) {
            assets.update { ArrayList<Asset>().apply { add(mainAsset) } }
        } else {
            assets.update { ArrayList() }
        }
    }

    fun sibiling(offset: Int) {
        if (assets.size == 0) {
            return
        }

        assets.currentIndex

        var newIndex = (assets.currentIndex + offset) % assets.size
        if (newIndex < 0) {
            newIndex += assets.size
        }

        assets.update(newIndex)
    }
}