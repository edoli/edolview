package kr.edoli.edolview

import com.badlogic.gdx.Gdx
import com.badlogic.gdx.math.Vector2
import kr.edoli.edolview.geom.Point2D
import kr.edoli.edolview.image.*
import kr.edoli.edolview.res.Asset
import kr.edoli.edolview.res.ClipboardAsset
import kr.edoli.edolview.res.ListAsset
import kr.edoli.edolview.shader.ViewerShaderBuilder
import kr.edoli.edolview.util.*
import kr.edoli.edolview.util.Observable
import org.opencv.core.Mat
import org.opencv.core.Rect
import rx.subjects.PublishSubject
import java.util.*
import kotlin.math.min

/**
 * Created by daniel on 16. 10. 2.
 */
object ImContext {
    enum class AssetType {
        File, URL, Content
    }

    private val preferences = Gdx.app.getPreferences("EdolView")

    val args = ObservableValue(arrayOf<String>(), "Args")

    val mainImage = ObservableValue<Mat?>(null, "Main image")
    val mainAsset = ObservableValue<Asset?>(null, "Main asset")
    val mainTitle = ObservableValue("", "Main file name")
    val mainAssetNavigator = Observable<Int>("Main file navigation")

    val recentAssets = ObservableValue(emptyList<Asset>(), "Recently opened files")

    val autoRefresh = ObservableValue(false, "Auto refresh")

    val mainImageSpec = ObservableValue<ImageSpec?>(null, "Image spec")
    val marqueeImage = ObservableLazyValue<Mat?>(null, "Marqueed image")

    val cursorPosition = ObservableValue(Point2D(0.0, 0.0), "Cursor position")
    val cursorRGB = ObservableValue(doubleArrayOf(), "Cursor RGB")

    val marqueeBox = ObservableValue(Rect(), "Marquee box") { rect ->
        mainImage.get()?.let { image ->
            val imageWidth = image.width()
            val imageHeight = image.height()

            val width = min(rect.width, imageWidth - rect.x)
            val height = min(rect.height, imageHeight - rect.y)

            rect.width = width
            rect.height = height
        }
        rect
    }
    val marqueeBoxActive = ObservableValue(false, "Marquee box active")
    val marqueeBoxRGB = ObservableValue(doubleArrayOf(), "Marquee box RGB")

    val zoom = ObservableValue(1f, "Zoom level")
    val zoomCenter = ObservableValue<Vector2?>(null, "Zoom center")
    val rotation = ObservableValue(0.0, "Rotation")

    // Display profile
    val viewerShaderBuilder = ViewerShaderBuilder()
    val viewerShader = ObservableValue(viewerShaderBuilder.getRGB("color"), "Viewer shader")
    val enableDisplayProfile = ObservableValue(true, "Enable display profile")
    val smoothing = ObservableValue(false, "Smoothing")
    val normalize = ObservableValue(false, "Normalize")
    val inverse = ObservableValue(false, "Inverse")

    val displayMin = ObservableValue(0.0f, "Display min")
    val displayMax = ObservableValue(1.0f, "Display max")

    val imageMinMax = ObservableLazyValue(Pair(0.0, 1.0), "Image min max")

    val imageExposure = ObservableValue(0.0f, "Exposure")
    val imageOffset = ObservableValue(0.0f, "Offset")
    val imageGamma = ObservableValue(1.0f, "Gamma")
    val imageMonoColormap = ObservableList(ViewerShaderBuilder.getColormapNames("mono"), name = "Colormap")
    val imageRGBColormap = ObservableList(ViewerShaderBuilder.getColormapNames("rgb"), name = "Colormap")
    val visibleChannel = ObservableList(listOf(0), name = "Visible channel")

    val frameInterval = ObservableValue(1, "Frame interval")
    val frameSpeed = ObservableValue(0.0f, "Frame speed")
    val frameControl = FrameControl()

    // UIgetScalingFactor
    val uiScale = ObservableValue(Platform.getScalingFactor(), "UI scaling factor")

    val isShowBackground = ObservableValue(false, "Show background")
    val isShowCrosshair = ObservableValue(true, "Show crosshair")
    val isShowController = ObservableValue(true, "Show controller")
    val isShowFileInfo = ObservableValue(false, "Show file info")
    val isShowStatusBar = ObservableValue(true, "Show status bar")

    val centerCursor = PublishSubject.create<Boolean>()
    val centerImage = PublishSubject.create<Boolean>()
    val fitImage = PublishSubject.create<Boolean>()
    val centerSelection = PublishSubject.create<Boolean>()
    val fitSelection = PublishSubject.create<Boolean>()

    val listAsset = ListAsset()

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

        mainAsset.subscribe(this, "Update asset") { asset ->
            if (asset == null) {
                mainImageSpec.update(null)
            } else {
                val spec = asset.retrieveImageSpec()
                spec?.normalize()

                mainImageSpec.update(spec)
                mainTitle.update(asset.name)

                // Add to recent files
                if (spec != null && asset.shouldAddToRecentAssets) {
                    val list = recentAssets.get().toMutableList()
                    val name = asset.name

                    // TODO: resolve when multiple assets have same name
                    val assetInList = list.firstOrNull { it.name == name }
                    if (assetInList != null) {
                        list.remove(assetInList)
                    }
                    list.add(asset)
                    while (list.size > 15) {
                        list.removeFirst()
                    }
                    recentAssets.update(list)
                }
            }
            
        }

        mainImageSpec.subscribe(this, "Update image spec") { spec ->
            if (spec != null) {
                if (!spec.isNormalized) {
                    spec.normalize()
                }

                imageMinMax.update {
                    spec.mat.minMax()
                }

                mainImage.update(spec.mat)

                updateCursorColor()
                marqueeBoxRGB.update(MarqueeUtils.boxMeanColor())
            }
        }

        mainImage.subscribe(this, "Update marquee") { mat ->
            cursorRGB.update(doubleArrayOf())
            marqueeBoxRGB.update(doubleArrayOf())

            if (mat != null) {
                val numChannels = mat.channels()
                val channelIndex = min(visibleChannel.currentIndex, numChannels)
                visibleChannel.update(IntRange(0, mat.channels()).toList(), channelIndex)

                val box = marqueeBox.get()
                box.x = box.x.clamp(0, mat.width())
                box.y = box.y.clamp(0, mat.height())
                box.width = box.width.clamp(0, mat.width() - box.x)
                box.height = box.height.clamp(0, mat.height() - box.y)

                if (box.width > 0 && box.height > 0) {
                    marqueeImage.update { mat[box] }
                    marqueeBoxRGB.update(MarqueeUtils.boxMeanColor())
                }
            }
        }

        mainAssetNavigator.subscribe(this, "Image navigation") { shift ->
            if (shift == 1) {
                mainAsset.update(mainAsset.get()?.next())
            } else {
                mainAsset.update(mainAsset.get()?.prev())
            }
        }

        visibleChannel.subscribeValue(this, "Visible channel change") { channel ->
            updateCurrentShader(channel ?: 0)
        }

        imageMonoColormap.subscribeValue(this, "Colormap change") { colormapName ->
            updateCurrentShader()
        }

        imageRGBColormap.subscribeValue(this, "Colormap change") { colormapName ->
            updateCurrentShader()
        }

        cursorPosition.subscribe(this, "Update mouse position and RGB") {
            updateCursorColor()
        }

        marqueeBox.subscribe(this, "Update marquee image and RGB") {
            val mainImage = mainImage.get()
            if (it.width > 0 && it.height > 0 && mainImage != null) {
                marqueeImage.update { mainImage[it] }
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
                val delta = newTime - current
                frameControl.elapse(delta / 1000f / 1000f / 1000f)
                current = newTime
            }
        }, 0, 10)
    }

    fun updateCurrentShader(channel: Int = visibleChannel.get() ?: 0) {
        if (channel != 0 || mainImage.get()?.channels() == 1) {
            val monoColormap = imageMonoColormap.get()
            if (monoColormap != null) {
                viewerShader.update(viewerShaderBuilder.getMono(monoColormap))
            }
        } else {
            val rgbColormap = imageRGBColormap.get()
            if (rgbColormap != null) {
                viewerShader.update(viewerShaderBuilder.getRGB(rgbColormap))
            }
        }
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
        // preferences.sync()
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

    fun loadDisplayProfile() {

    }

    fun saveDisplayProfile() {

    }

    fun loadFromClipboard() {
        ClipboardUtils.processClipboard({
            mainAsset.update(ClipboardAsset())
        }, {
            val file = ClipboardUtils.getFileList()[0]
            mainAsset.update(Asset.fromUri(file.absolutePath))
        }, {
            mainAsset.update(Asset.fromUri(ClipboardUtils.getString()))
        })
    }

    fun refreshAsset() {
        mainAsset.update(mainAsset.get())
    }

    fun addAssetsFromFiles(files: Array<String>) {
        if (mainAsset.get() == listAsset) {
            listAsset.assets.update {
                val newList = it.toMutableList()
                for (file in files) {
                    newList.add(Asset.fromUri(file))
                }
                newList
            }
        } else {
            mainAsset.update(Asset.fromUri(files[0]))
        }
    }
}