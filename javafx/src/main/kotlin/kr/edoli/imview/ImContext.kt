package kr.edoli.imview

import kr.edoli.imview.geom.Point2D
import kr.edoli.imview.image.*
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.ObservableValue
import kr.edoli.imview.util.FileManager
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
    val mainFile = ObservableValue(File("test.jpg"))

    val imageSpec = NullableObservableValue<ImageSpec>(null)
    val selectedImage = NullableObservableValue<Mat>(null)


    val cursorPosition = ObservableValue(Point2D(0.0, 0.0))
    val cursorRGB = ObservableValue(doubleArrayOf())
    val selectBox = ObservableValue(Rect())
    val selectBoxActive = ObservableValue(false)
    val selectBoxRGB = ObservableValue(doubleArrayOf())
    val zoomLevel = ObservableValue(0)
    val zoomCenter = ObservableValue(Point2D(0.0, 0.0))
    val rotation = ObservableValue(0.0)

    val isShowCrosshair = ObservableValue(true)
    val isShowConroller = ObservableValue(true)
    val isShowInfo = ObservableValue(false)

    val enableProfile = ObservableValue(true)
    val normalize = ObservableValue(false)
    val smoothing = ObservableValue(false)

    val imageContrast = ObservableValue(1.0f, "Contrast")
    val imageBrightness = ObservableValue(0.0f, "Brightness")
    val imageGamma = ObservableValue(1.0f, "Gamma")

    val frameSpeed = ObservableValue(0.0f)

    val centerImage = PublishSubject.create<Boolean>()

    val fileManager = FileManager()

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
                    imageSpec.update(spec)
                    val normalized = ImageStore.normalize(mat)
                    mainImage.update(normalized)

                    updateCursorColor()
                    selectBoxRGB.update(MarqueeUtils.boxMeanColor())
                }
            }
        }

        mainImage.subscribe {
            cursorRGB.update(doubleArrayOf())
            selectBoxRGB.update(doubleArrayOf())
        }

        cursorPosition.subscribe(this) {
            updateCursorColor()
        }

        selectBox.subscribe(this) {
            val mainImage = mainImage.get()
            if (it.width > 0 && it.height > 0 && mainImage != null) {
                selectedImage.update(mainImage[it])
                selectBoxRGB.update(MarqueeUtils.boxMeanColor())
            }
        }

        isShowCrosshair.subscribe { savePreferences() }
        isShowConroller.subscribe { savePreferences() }
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
        val mainImage = ImContext.mainImage.get()
        val point = cursorPosition.get().cvPoint
        if (mainImage != null && mainImage.contains(point)) {
            val color = mainImage[point]
            ImContext.cursorRGB.update(color)
        }
    }

    fun loadPreferences() {
        preferences.sync()
        isShowCrosshair.update(preferences.getBoolean("isShowCrossHair", false))
        isShowConroller.update(preferences.getBoolean("isShowConroller", false))
        isShowInfo.update(preferences.getBoolean("isShowInfo", false))
    }

    fun savePreferences() {
        preferences.putBoolean("isShowCrossHair", isShowCrosshair.get())
        preferences.putBoolean("isShowConroller", isShowConroller.get())
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