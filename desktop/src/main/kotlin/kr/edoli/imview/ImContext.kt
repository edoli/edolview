package kr.edoli.imview

import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.*
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.ui.Colormap
import kr.edoli.imview.util.*
import org.opencv.core.Mat
import org.opencv.core.Rect
import rx.subjects.PublishSubject
import java.io.File
import java.util.*
import java.util.prefs.Preferences
import kotlin.math.min

/**
 * Created by daniel on 16. 10. 2.
 */
object ImContext {
    private val preferences = Preferences.userRoot().node("ImView")

    val args = ObservableValue(arrayOf<String>(), "Args")

    val mainImage = ObservableValue<Mat?>(null, "Main image")
    val mainFile = ObservableValue(File("EdolView"), "Main file")
    val mainFileName = ObservableValue("", "Main File name")
    val mainFileDirectory = ObservableValue("", "Main File directory")

    val mainImageSpec = ObservableValue<ImageSpec?>(null, "Image spec")
    val marqueeImage = ObservableValue<Mat?>(null, "Marqueed image")

    val cursorPosition = ObservableValue(Point2D(0.0, 0.0), "Cursor position")
    val cursorRGB = ObservableValue(doubleArrayOf(), "Cursor RGB")

    val marqueeBox = ObservableValue(Rect(), "Marquee box", { rect ->
        mainImage.get()?.let { image ->
            val imageWidth = image.width()
            val imageHeight = image.height()

            val width = min(rect.width, imageWidth - rect.x)
            val height = min(rect.height, imageHeight - rect.y)

            rect.width = width
            rect.height = height
        }
        rect
    })
    val marqueeBoxActive = ObservableValue(false, "Marquee box active")
    val marqueeBoxRGB = ObservableValue(doubleArrayOf(), "Marquee box RGB")

    val zoomLevel = ObservableValue(0, "Zoom level")
    val zoomCenter = ObservableValue(Point2D(0.0, 0.0), "Zoom center")
    val rotation = ObservableValue(0.0, "Rotation")

    // Display profile
    val viewerShaderBuilder = ViewerShaderBuilder()
    val viewerShader = ObservableValue(viewerShaderBuilder.build(), "Viewer shader")
    val enableDisplayProfile = ObservableValue(true, "Enable display profile")
    val normalize = ObservableValue(false, "Normalize")
    val smoothing = ObservableValue(false, "Smoothing")

    val displayMin = ObservableValue(0.0f, "Display Min")
    val displayMax = ObservableValue(1.0f, "Display Max")

    val textureMin = ObservableValue(0.0f, "Image Min")
    val textureMax = ObservableValue(1.0f, "Image Max")

    val imageContrast = ObservableValue(1.0f, "Contrast")
    val imageBrightness = ObservableValue(0.0f, "Brightness")
    val imageGamma = ObservableValue(1.0f, "Gamma")
    val imageColormap = ObservableList(Colormap.values().toList(), name = "Colormap")
    val visibleChannel = ObservableList(listOf(0), name = "Visible channel")

    val frameInterval = ObservableValue(1, "Frame interval")
    val frameSpeed = ObservableValue(0.0f, "Frame speed")
    val frameControl = FrameControl()

    // UI
    val isShowCrosshair = ObservableValue(true, "Show crosshair")
    val isShowController = ObservableValue(true, "Show controller")
    val isShowFileInfo = ObservableValue(false, "Show file info")

    val centerCursor = PublishSubject.create<Boolean>()
    val centerImage = PublishSubject.create<Boolean>()
    val fitImage = PublishSubject.create<Boolean>()
    val centerSelection = PublishSubject.create<Boolean>()
    val fitSelection = PublishSubject.create<Boolean>()

    val fileManager = FileManager()

    val isValidMarquee: Boolean
        get() {
            mainImage.get()?.let { mat ->
                val marqueeRect = marqueeBox.get()
                return (marqueeRect.x >= 0 && marqueeRect.y >= 0
                        && marqueeRect.width > 0
                        && marqueeRect.height > 0
                        && (marqueeRect.x + marqueeRect.width) <= mat.width()
                        && (marqueeRect.y + marqueeRect.height) <= mat.height())
            }
            return false
        }

    init {
        loadPreferences()

        mainFile.subscribe(this, "Update image") { file ->
            if (file.isDirectory) {
                fileManager.setFile(file)
                frameSpeed.update(5.0f)
                nextImage()
            } else {
                mainFileName.update(file.name)
                mainFileDirectory.update(file.absoluteFile.parent)
                fileManager.setFile(file)
                val spec = ImageStore.get(file)
                val mat = spec.mat
                if (!mat.empty()) {
                    mainImageSpec.update(spec)
                    val normalized = ImageStore.normalize(mat)
                    mainImage.update(normalized)

                    updateCursorColor()
                    marqueeBoxRGB.update(MarqueeUtils.boxMeanColor())
                }
            }
        }

        mainImage.subscribe(this, "Update marquee") { mat ->
            cursorRGB.update(doubleArrayOf())
            marqueeBoxRGB.update(doubleArrayOf())

            if (mat != null) {
                val numChannels = mat.channels()
                val channelIndex = min(visibleChannel.currentIndex, numChannels)
                visibleChannel.update(IntRange(0, mat.channels()).toList(), channelIndex)
            }
        }

        cursorPosition.subscribe(this, "Update mouse position and RGB") {
            updateCursorColor()
        }

        marqueeBox.subscribe(this, "Update marquee image and RGB") {
            val mainImage = mainImage.get()
            if (it.width > 0 && it.height > 0 && mainImage != null) {
                marqueeImage.update(mainImage[it])
                marqueeBoxRGB.update(MarqueeUtils.boxMeanColor())
            }
        }

        isShowCrosshair.subscribe(this, "Preference") { savePreferences() }
        isShowController.subscribe(this, "Preference") { savePreferences() }
        isShowFileInfo.subscribe(this, "Preference") { savePreferences() }

        Timer().schedule(object : TimerTask() {
            var current = System.nanoTime()
            override fun run() {
                val newTime = System.nanoTime()
                var delta = newTime - current
                frameControl.elapse(delta / 1000f / 1000f / 1000f)
                current = newTime
            }
        }, 0, 10)
    }

    fun updateCursorColor() {
        val mainImage = mainImage.get()
        val point = cursorPosition.get().cvPoint
        if (mainImage != null && mainImage.contains(point)) {
            val color = mainImage[point]
            cursorRGB.update(color)
        }
    }

    fun loadPreferences() {
        preferences.sync()
        isShowCrosshair.update(preferences.getBoolean("isShowCrossHair", false))
        isShowController.update(preferences.getBoolean("isShowConroller", false))
        isShowFileInfo.update(preferences.getBoolean("isShowInfo", false))
    }

    fun savePreferences() {
        preferences.putBoolean("isShowCrossHair", isShowCrosshair.get())
        preferences.putBoolean("isShowConroller", isShowController.get())
        preferences.putBoolean("isShowInfo", isShowFileInfo.get())
        preferences.flush()
    }

    fun nextImage() {
        val nextFile = fileManager.next(frameInterval.get())
        if (nextFile != null) {
            mainFile.update(nextFile)
        }
    }

    fun prevImage() {
        val prevFile = fileManager.prev(frameInterval.get())
        if (prevFile != null) {
            mainFile.update(prevFile)
        }
    }
}