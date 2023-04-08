package kr.edoli.edolview

import com.badlogic.gdx.utils.Timer

object PeriodicTask : Timer.Task() {
    override fun run() {
        // check refresh
        if (ImContext.autoRefresh.get()) {
            val asset = ImContext.mainAsset.get()
            if (asset != null && asset.checkRefresh()) {
                ImContext.refreshAsset()
            }
        }
    }
}