package kr.edoli.edolview.ui.component

import com.badlogic.gdx.math.Vector2
import com.badlogic.gdx.scenes.scene2d.Group
import kr.edoli.edolview.ui.ImageViewer

class MarkerGroup(val imageViewer: ImageViewer): Group() {

    fun addMarkerLocal(marker: Marker, x: Float, y: Float) {
        val localPos = Vector2(x, y)
        imageViewer.localToImageCoordinates(localPos)

        addMarkerImage(marker, localPos.x, localPos.y)
    }

    fun addMarkerImage(marker: Marker, x: Float, y: Float) {
        marker.imageX = x
        marker.imageY = y
        marker.update()

        addActor(marker)
    }

    fun getMarkerAtLocal(x: Float, y: Float): Marker? {
        val distanceThreshold = 10.0f
        var minDist = distanceThreshold
        var target: Marker? = null

        for (child in children) {
            val marker = child as Marker
            val dist = Vector2(marker.x - x, marker.y - y).len()

            if (dist < minDist) {
                target = marker
                minDist = dist
            }
        }

        return target
    }

    fun update() {
        val localPos = Vector2()

        for (child in children) {
            val marker = child as Marker
            marker.update()
            localPos.set(marker.imageX, marker.imageY)
            imageViewer.imageToLocalCoordinates(localPos)
            marker.x = localPos.x
            marker.y = localPos.y
        }
    }
}