package kr.edoli.imview

import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.ImageSpec
import kr.edoli.imview.image.MarqueeUtils
import kr.edoli.imview.image.contains
import kr.edoli.imview.image.get
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.util.FileManager
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.ObservableValue
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

    val isShowCrosshair = ObservableValue(true, "Show crosshair")
    val isShowController = ObservableValue(true, "Show controller")
    val isShowInfo = ObservableValue(false, "Show info")

    val enableProfile = ObservableValue(true, "Enable profile")
    val normalize = ObservableValue(false, "Normalize")
    val smoothing = ObservableValue(false, "Smoothing")

    val imageContrast = ObservableValue(1.0f, "Contrast")
    val imageBrightness = ObservableValue(0.0f, "Brightness")
    val imageGamma = ObservableValue(1.0f, "Gamma")

    val frameSpeed = ObservableValue(0.0f, "Frame speed")

    val centerImage = PublishSubject.create<Boolean>()
    val fitImage = PublishSubject.create<Boolean>()

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

        mainImage.subscribe {
            cursorRGB.update(doubleArrayOf())
            marqueeBoxRGB.update(doubleArrayOf())
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
        isShowInfo.subscribe { savePreferences() }

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
        isShowInfo.update(preferences.getBoolean("isShowInfo", false))
    }

    fun savePreferences() {
        preferences.putBoolean("isShowCrossHair", isShowCrosshair.get())
        preferences.putBoolean("isShowConroller", isShowController.get())
        preferences.putBoolean("isShowInfo", isShowInfo.get())
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