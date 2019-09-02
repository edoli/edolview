package kr.edoli.imview

import javafx.animation.AnimationTimer
import javafx.geometry.Point2D
import kr.edoli.imview.image.*
import kr.edoli.imview.store.ImageStore
import kr.edoli.imview.util.NullableObservableValue
import kr.edoli.imview.util.ObservableValue
import kr.edoli.imview.util.PathManager
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
    val mainPath = ObservableValue("")

    val imageSpec = NullableObservableValue<ImageSpec>(null)
    val processedImage = NullableObservableValue<Mat>(null)
    val selectedImage = NullableObservableValue<Mat>(null)


    val cursorPosition = ObservableValue(Point2D(0.0, 0.0))
    val cursorRGB = ObservableValue(doubleArrayOf())
    val selectBox = ObservableValue(Rect())
    val selectBoxActive = ObservableValue(false)
    val selectBoxRGB = ObservableValue(doubleArrayOf())
    val zoomRatio = ObservableValue(1.0)
    val zoomCenter = ObservableValue(Point2D(0.0, 0.0))
    val rotation = ObservableValue(0.0)

    val comparisonMode = ObservableValue(ComparisonMode.Diff)
    val comparisonMetric = ObservableValue(ComparisonMetric.PSNR)

    val isShowCrosshair = ObservableValue(true)
    val isShowConroller = ObservableValue(true)
    val isShowInfo = ObservableValue(false)

    val enableProfile = ObservableValue(true)
    val normalize = ObservableValue(false)
    val imageContrast = ObservableValue(0.0)
    val imageBrightness = ObservableValue(0.0)
    val imageGamma = ObservableValue(1.0)

    val frameSpeed = ObservableValue(0.0)

    val centerImage = PublishSubject.create<Boolean>()

    val pathManager = PathManager()

    init {
        loadPreferences()

        mainPath.update("test.jpg")

        mainPath.subscribe {
            val file = File(it)
            if (file.isDirectory) {
                pathManager.setPath("$it/.")
                frameSpeed.update(5.0)
                nextImage()
            } else {
                pathManager.setPath(it)
                val spec = ImageStore.get(File(it))
                val mat = spec.mat
                if (!mat.empty()) {
                    imageSpec.update(spec)
                    val normalized = ImageStore.normalize(mat)
                    mainImage.update(normalized)

                    updateCursorColor()
                    selectBoxRGB.update(SelectBoxUtils.selectBoxMeanColor())
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
                selectBoxRGB.update(SelectBoxUtils.selectBoxMeanColor())
            }
        }

        isShowCrosshair.subscribe { savePreferences() }
        isShowConroller.subscribe { savePreferences() }
        isShowInfo.subscribe { savePreferences() }

        object : AnimationTimer() {
            var lastTime = 0L

            override fun handle(now: Long) {
                val fps = frameSpeed.get()
                if (fps != 0.0) {
                    val waitTime = 1 / fps * 1000 * 1000 * 1000
                    if (now > lastTime + waitTime) {
                        nextImage()
                        lastTime = now
                    }
                }

            }
        }.start()
    }

    fun updateCursorColor() {
        val mainImage = ImContext.mainImage.get()
        val point = cursorPosition.get().cv
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
        val nextPath = pathManager.next()
        if (nextPath != null) {
            mainPath.update(nextPath)
        }
    }

    fun prevImage() {
        val prevPath = pathManager.prev()
        if (prevPath != null) {
            mainPath.update(prevPath)
        }
    }
}