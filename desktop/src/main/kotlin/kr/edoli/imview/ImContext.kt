package kr.edoli.imview

import com.badlogic.gdx.Gdx
import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.ImageSpec
import kr.edoli.imview.image.MarqueeUtils
import kr.edoli.imview.image.contains
import kr.edoli.imview.image.get
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.ui.Colormap
import kr.edoli.imview.util.*
import org.opencv.core.Mat
import org.opencv.core.Rect
import rx.subjects.PublishSubject
import java.io.File
import java.util.prefs.Preferences

/**
 * Created by daniel on 16. 10. 2.
 */
object ImContext {
    private val preferences = Preferences.userRoot().node("ImView")

    val args = ObservableValue(arrayOf<String>())

    val mainImage = NullableObservableValue<Mat>(null)
    val mainFile = ObservableValue(File("test.png"))
    val mainFileName = ObservableValue("", "File name")
    val mainFileDirectory = ObservableValue("", "File directory")

    val mainImageSpec = NullableObservableValue<ImageSpec>(null)
    val marqueeImage = NullableObservableValue<Mat>(null)


    val cursorPosition = ObservableValue(Point2D(0.0, 0.0))
    val cursorRGB = ObservableValue(doubleArrayOf())

    val marqueeBox = ObservableValue(Rect())
    val marqueeBoxActive = ObservableValue(false)
    val marqueeBoxRGB = ObservableValue(doubleArrayOf())

    val zoomLevel = ObservableValue(0, "Zoom level")
    val zoomCenter = ObservableValue(Point2D(0.0, 0.0))
    val rotation = ObservableValue(0.0)

    // Display profile
    val enableDisplayProfile = ObservableValue(true, "Enable display profile")
    val normalize = ObservableValue(false, "Normalize")
    val smoothing = ObservableValue(false, "Smoothing")

    val imageContrast = ObservableValue(1.0f, "Contrast")
    val imageBrightness = ObservableValue(0.0f, "Brightness")
    val imageGamma = ObservableValue(1.0f, "Gamma")
    val imageColormap = ObservableList(Colormap.values().toList(), name ="Colormap")
    val visibleChannel = ObservableList(listOf(0), name = "Visible channel")

    val frameSpeed = ObservableValue(0.0f, "Frame speed")

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

        mainFile.subscribe { file ->
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

        mainImage.subscribe { mat ->
            cursorRGB.update(doubleArrayOf())
            marqueeBoxRGB.update(doubleArrayOf())

            if (mat != null) {
                visibleChannel.update(IntRange(0, mat.channels()).toList())
            }
        }

        cursorPosition.subscribe(this) {
            updateCursorColor()
        }

        marqueeBox.subscribe(this) {
            val mainImage = mainImage.get()
            if (it.width > 0 && it.height > 0 && mainImage != null) {
                marqueeImage.update(mainImage[it])
                marqueeBoxRGB.update(MarqueeUtils.boxMeanColor())
            }
        }

        isShowCrosshair.subscribe { savePreferences() }
        isShowController.subscribe { savePreferences() }
        isShowFileInfo.subscribe { savePreferences() }

        // object : AnimationTimer() {
        //     var lastTime = 0L
        //
        //     override fun handle(now: Long) {
        //         val fps = frameSpeed.get()
        //         if (fps != 0.0) {
        //             val waitTime = 1 / fps * 1000 * 1000 * 1000
        //             if (now > lastTime + waitTime) {
        //                 nextImage()
        //                 lastTime = now
        //             }
        //         }
        //
        //     }
        // }.start()
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
        val nextFile = fileManager.next()
        if (nextFile != null) {
            mainFile.update(nextFile)
        }
    }

    fun prevImage() {
        val prevFile = fileManager.prev()
        if (prevFile != null) {
            mainFile.update(prevFile)
        }
    }
}