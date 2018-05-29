package kr.edoli.imview.ui

import javafx.geometry.Point2D
import javafx.scene.effect.ColorAdjust
import javafx.scene.image.Image
import javafx.scene.input.KeyCode
import javafx.scene.layout.Pane
import javafx.scene.paint.Color
import javafx.scene.paint.Paint
import javafx.scene.shape.Line
import javafx.scene.shape.Rectangle
import javafx.scene.transform.NonInvertibleTransformException
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.ImageConvert
import kr.edoli.imview.image.ImageProc
import kr.edoli.imview.image.set
import org.kordamp.ikonli.javafx.FontIcon
import org.kordamp.ikonli.materialdesign.MaterialDesign
import org.opencv.core.Rect
import tornadofx.add
import tornadofx.imageview


class ImageViewer : Pane() {

    enum class DragState {
        Move, Select
    }

    private var mousePosX = 0.0
    private var mousePosY = 0.0
    private var mousePressedPosX = 0.0
    private var mousePressedPosY = 0.0
    private var dragState = DragState.Move

    // image
    var image: Image? = null
        set(image) {
            field = image
            imageView.image = image
        }

    // children widgets
    private val imageRegion = Pane()
    private val imageView = PixelatedImageView()
    private val contextMenu = ImContextMenu()
    private val zoomButton = CircleButton(12.0).apply { graphic = FontIcon(MaterialDesign.MDI_MAGNIFY) }

    // rectangle for seleciton
    private val geometricRect = Rect()
    private val visualizeRect = Rectangle()
    private var rectActive = false
    private val crossHairLineA = Line().apply { stroke = Color.LIGHTGREEN }
    private val crossHairLineB = Line().apply { stroke = Color.LIGHTGREEN }

    private var lastUpdateTime = 0

    init {
        ImContext.mainImage.subscribe {
            updateImage()
        }

        ImContext.imageContrast.subscribe {
            updateImage()
        }

        ImContext.imageBrightness.subscribe {
            updateImage()
        }

        ImContext.imageGamma.subscribe {
            updateImage()
        }

        ImContext.zoomRatio.subscribe(this) {
            val center = ImContext.zoomCenter.get()
            zoom(it, Point2D(center.x, center.y))
        }

        ImContext.isShowCrosshair.subscribe(this) {
            crossHairLineA.isVisible = it
            crossHairLineB.isVisible = it
        }

        ImContext.centerImage.subscribe {
            centerImage()
        }

        add(imageRegion)
        add(crossHairLineA)
        add(crossHairLineB)
        add(Pane(visualizeRect))
        add(Pane(zoomButton))

        visualizeRect.fill = Color.TRANSPARENT

        val colorAdjust = ColorAdjust()
        // ImContext.imageBrightness.subscribe { colorAdjust.brightness = it }
        // ImContext.imageContrast.subscribe { colorAdjust.contrast = it }

        imageView.isPreserveRatio = true
        imageView.isSmooth = false
        imageView.effect = colorAdjust
        imageRegion.add(imageView)

        updateVisualizeRect()

        // Event listeners
        setOnMouseMoved { e ->
            mousePosX = e.sceneX
            mousePosY = e.sceneY

            updateCursor()
        }

        setOnMousePressed { e ->
            mousePosX = e.sceneX
            mousePosY = e.sceneY

            mousePressedPosX = mousePosX
            mousePressedPosY = mousePosY

            dragState = DragState.Move
            if (e.isShiftDown && !e.isControlDown) {
                dragState = DragState.Select
            }

            requestFocus()

            contextMenu.hide()
        }

        setOnContextMenuRequested { e ->
            contextMenu.show(this, e.screenX, e.screenY)
        }

        setOnKeyPressed { e ->
            if (e.code == KeyCode.ESCAPE) {
                rectActive = false
                updateVisualizeRect()
            } else if (e.code == KeyCode.A && e.isControlDown) {
                rectActive = true
                selectAll()
                updateVisualizeRect()
            }
        }

        setOnZoom { e ->
            try {
                val pointOnImage = imageRegion.localToSceneTransform.inverseTransform(e.sceneX, e.sceneY)
                zoom(e.zoomFactor, pointOnImage)
            } catch (e1: NonInvertibleTransformException) {
                e1.printStackTrace()
            }
        }

        setOnScroll { e ->
            try {
                val pointOnImage = imageRegion.localToSceneTransform.inverseTransform(e.sceneX, e.sceneY)
                val zoomRatio = (if (e.deltaY > 0) 1.1 else 1.0/1.1) * ImContext.zoomRatio.get()

                ImContext.zoomCenter.update(pointOnImage)
                ImContext.zoomRatio.update(zoomRatio)
            } catch (e1: NonInvertibleTransformException) {
                e1.printStackTrace()
            }
        }

        setOnMouseDragged { e ->
            val mouseDeltaX = e.sceneX - mousePosX
            val mouseDeltaY = e.sceneY - mousePosY
            mousePosX = e.sceneX
            mousePosY = e.sceneY

            when (dragState) {
                DragState.Move -> {
                    imageRegion.translateX = imageRegion.translateX + mouseDeltaX
                    imageRegion.translateY = imageRegion.translateY + mouseDeltaY
                }
                DragState.Select -> {
                    rectActive = true
                    resizeGeometricRect()
                }
            }
            updateVisualizeRect()
            updateCursor()
        }

        zoomButton.setOnMouseClicked {
            val centerX = geometricRect.x + geometricRect.width / 2.0
            val centerY = geometricRect.y + geometricRect.height / 2.0
            val centerPoint = Point2D(centerX, centerY)

            val parentPane = parent as Pane
            val widthRatio = parentPane.width / geometricRect.width
            val heightRatio = parentPane.height / geometricRect.height
            val zoomRatio = Math.min(widthRatio, heightRatio)

            centerPosition(centerPoint)
            ImContext.zoomCenter.update(centerPoint)
            ImContext.zoomRatio.update(zoomRatio)
        }
    }

    fun updateImage() {
        val mat = ImContext.mainImage.get()
        if (mat != null) {
            val newImage = ImageProc.process(mat)
            if (newImage != null) {
                image = newImage
            }
        }
    }

    fun centerImage() {
        val center = Point2D(imageRegion.width / 2, imageRegion.height / 2)
        centerPosition(center)
    }

    private fun resizeGeometricRect() {
        val image = this.image

        if (image != null) {
            val x1 = minOf(mousePosX, mousePressedPosX)
            val y1 = minOf(mousePosY, mousePressedPosY)
            val x2 = maxOf(mousePosX, mousePressedPosX)
            val y2 = maxOf(mousePosY, mousePressedPosY)

            val leftTop = imageRegion.sceneToLocal(x1, y1)
            val rightBottom = imageRegion.sceneToLocal(x2, y2)

            var minX = leftTop.x.toInt()
            var minY = leftTop.y.toInt()
            var maxX = rightBottom.x.toInt()
            var maxY = rightBottom.y.toInt()

            minX = Math.max(minX, 0)
            minY = Math.max(minY, 0)
            maxX = Math.min(maxX, image.width.toInt() - 1)
            maxY = Math.min(maxY, image.height.toInt() - 1)

            geometricRect.x = minX
            geometricRect.y = minY
            geometricRect.width = maxX - minX + 1
            geometricRect.height = maxY - minY + 1

            ImContext.selectBox.update { it.set(geometricRect) }
        }
    }

    private fun selectAll() {
        val image = this.image

        if (image != null) {
            geometricRect.x = 0
            geometricRect.y = 0
            geometricRect.width = image.width.toInt()
            geometricRect.height = image.height.toInt()

            ImContext.selectBox.update { it.set(geometricRect) }

        }

    }

    private fun updateVisualizeRect() {
        if (!rectActive) {
            visualizeRect.isVisible = false
            zoomButton.isVisible = false
            ImContext.selectBoxActive.update(false)
            return
        }
        zoomButton.isVisible = true
        visualizeRect.isVisible = true
        ImContext.selectBoxActive.update(true)

        val leftTop = imageRegion.localToParent(geometricRect.x.toDouble(), geometricRect.y.toDouble())
        val rightBottom = imageRegion.localToParent(
                (geometricRect.x + geometricRect.width).toDouble(),
                (geometricRect.y + geometricRect.height).toDouble())

        visualizeRect.x = Math.round(leftTop.x).toDouble() + 0.5
        visualizeRect.y = Math.round(leftTop.y).toDouble() + 0.5
        visualizeRect.width = Math.round(rightBottom.x - visualizeRect.x).toDouble()
        visualizeRect.height = Math.round(rightBottom.y - visualizeRect.y).toDouble()

        when (dragState) {
            DragState.Move -> {}
            DragState.Select -> {
                visualizeRect.stroke = Paint.valueOf("#00a8ff")
            }
        }

        zoomButton.layoutX = visualizeRect.x + visualizeRect. width / 2 - zoomButton.layoutBounds.width / 2
        zoomButton.layoutY = visualizeRect.y - 32
    }

    private fun updateCursor() {
        val cursorPosition = imageRegion.sceneToLocal(mousePosX, mousePosY)
        ImContext.cursorPosition.update(cursorPosition)

        val mouseViewerPos = sceneToLocal(mousePosX, mousePosY)
        crossHairLineA.startX = mouseViewerPos.x + 0.5
        crossHairLineA.endX = mouseViewerPos.x + 0.5
        crossHairLineA.startY = 0.0
        crossHairLineA.endY = height

        crossHairLineB.startX = 0.0
        crossHairLineB.endX = width
        crossHairLineB.startY = mouseViewerPos.y + 0.5
        crossHairLineB.endY = mouseViewerPos.y + 0.5

    }

    private fun centerPosition(pointOnImage: Point2D) {
        val parentPane = parent as Pane

        val centerX = pointOnImage.x - imageRegion.boundsInLocal.width / 2
        val centerY = pointOnImage.y - imageRegion.boundsInLocal.height / 2

        val posX = centerX * imageRegion.scaleX + imageRegion.boundsInLocal.width / 2
        val posY = centerY * imageRegion.scaleY + imageRegion.boundsInLocal.height / 2

        imageRegion.translateX = parentPane.width / 2 - posX
        imageRegion.translateY = parentPane.height / 2 - posY

        updateCursor()
        updateVisualizeRect()
    }

    private fun zoom(zoomRatio: Double, pointOnImage: Point2D) {
        val zoomFactor = zoomRatio / imageRegion.scaleX

        val currentX = pointOnImage.x
        val currentY = pointOnImage.y

        val currentDistanceFromCenterX = currentX - imageRegion.boundsInLocal.width / 2
        val currentDistanceFromCenterY = currentY - imageRegion.boundsInLocal.height / 2

        val addScaleX = currentDistanceFromCenterX * zoomFactor
        val addScaleY = currentDistanceFromCenterY * zoomFactor

        val translationX = addScaleX - currentDistanceFromCenterX
        val translationY = addScaleY - currentDistanceFromCenterY

        imageRegion.translateX = imageRegion.translateX - translationX * imageRegion.scaleX
        imageRegion.translateY = imageRegion.translateY - translationY * imageRegion.scaleY

        imageRegion.scaleX = imageRegion.scaleX * zoomFactor
        imageRegion.scaleY = imageRegion.scaleY * zoomFactor

        updateCursor()
        updateVisualizeRect()
    }
}