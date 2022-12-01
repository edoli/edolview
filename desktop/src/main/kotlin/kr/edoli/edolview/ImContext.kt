package kr.edoli.edolview

import kr.edoli.edolview.geom.Point2D
import kr.edoli.edolview.image.*
import kr.edoli.edolview.shader.ViewerShaderBuilder
import kr.edoli.edolview.store.ImageStore
import kr.edoli.edolview.util.*
import kr.edoli.edolview.util.Observable
import org.opencv.core.Mat
import org.opencv.core.Rect
import rx.subjects.PublishSubject
import java.awt.image.BufferedImage
import java.io.File
import java.io.FileNotFoundException
import java.net.URL
import java.util.*
import java.util.prefs.Preferences
import kotlin.math.min

/**
 * Created by daniel on 16. 10. 2.
 */
object ImContext {
    private val preferences = Preferences.userRoot().node("EdolView")

    val args = ObservableValue(arrayOf<String>(), "Args")

    val mainImage = ObservableValue<Mat?>(null, "Main image")
    val mainPath = ObservableValue("EdolView", "Main path")
    val mainFile = ObservableValue<File?>(null, "Main file")
    val mainFileLastModified = ObservableValue(0L, "Main file last modified")
    val mainFileName = ObservableValue("", "Main file name")
    val mainFileDirectory = ObservableValue("", "Main file directory")

    val mainFileNavigator = Observable<Int>("Main file navigation")

    val recentFiles = ObservableValue(emptyList<String>(), "Recently opened files")

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

    val zoomLevel = ObservableValue(0, "Zoom level")
    val zoomCenter = ObservableValue(Point2D(0.0, 0.0), "Zoom center")
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

    val imageContrast = ObservableValue(1.0f, "Contrast")
    val imageBrightness = ObservableValue(0.0f, "Brightness")
    val imageGamma = ObservableValue(1.0f, "Gamma")
    val imageMonoColormap = ObservableList(ViewerShaderBuilder.getColormapNames("mono"), name = "Colormap")
    val imageRGBColormap = ObservableList(ViewerShaderBuilder.getColormapNames("rgb"), name = "Colormap")
    val visibleChannel = ObservableList(listOf(0), name = "Visible channel")

    val frameInterval = ObservableValue(1, "Frame interval")
    val frameSpeed = ObservableValue(0.0f, "Frame speed")
    val frameControl = FrameControl()

    // UI
    val uiScale = ObservableValue(getScalingFactor(), "UI scaling factor")

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

        val urlProtocols = arrayOf("http", "https", "ftp")

        mainPath.subscribe(this, "Update image") { path ->
            if (path == "clipboard") {
                // from clipboard
                val image = ClipboardUtils.getImage() as BufferedImage?
                if (image != null) {
                    val spec = ImageSpec(ImageConvert.bufferedToMat(image), 255.0, 8)
                    spec.normalize()
                    mainFile.update(null)
                    mainImageSpec.update(spec)
                    mainFileName.update("clipboard")
                    mainFileDirectory.update("")
                    fileManager.setFile(null)
                }
                return@subscribe
            }

            var isLocal = true
            if (":" in path) {
                val protocol = path.split(":")[0]
                if (protocol in urlProtocols) {
                    isLocal = false
                }
                if (protocol.count { it == '.' } == 3) {
                    isLocal = false
                }
            }
            if ("/" in path) {
                if (path.split("/")[0].count { it == '.' } == 3) {
                    isLocal = false
                }
            }

            if (isLocal) {
                // local file
                val file = File(path)

                if (!fileManager.isImageFile(file)) {
                    return@subscribe
                }

                if (file.isDirectory) {
                    fileManager.setFile(file)
                    mainFileNavigator.update(1)
                } else if (file.isFile) {
                    mainFile.update(file)
                    mainFileName.update(file.name)
                    mainFileDirectory.update(file.absoluteFile.parent)
                    fileManager.setFile(file)
                    val spec = ImageStore.get(file)
                    val mat = spec.mat
                    if (!mat.empty()) {
                        mainImageSpec.update(spec)
                    }
                } else {
                    // Not from file. Clear file manager
                    mainFile.update(null)
                    mainFileName.update(path)
                    mainFileDirectory.update("")
                    fileManager.setFile(null)
                }
            } else {
                // from url
                val url = URL(path)
                val bytes = try {
                    url.readBytes()
                } catch (e: FileNotFoundException) {
                    // url file not found
                    return@subscribe
                }
                val mat = ImageConvert.bytesToMat(bytes)

                if (mat.width() <= 0 || mat.height() <= 0) {
                    return@subscribe
                }

                val spec = ImageSpec(mat)
                spec.normalize()
                mainFile.update(null)
                mainImageSpec.update(spec)
                mainFileName.update(path)
                mainFileDirectory.update("")
                fileManager.setFile(null)
            }
        }

        mainFile.subscribe(this, "Update last modified when file updated") { file ->
            if (file != null) {
                mainFileLastModified.update(file.lastModified())
            } else {
                mainFileLastModified.update(0)
            }
        }

        mainFile.subscribe(this, "Check is file in same directory") { file ->
            if (file != null && !fileManager.isInSameDirectory(file)) {
                fileManager.reset()
            }
        }

        mainFile.subscribe(this, "Add to recent files") { file ->
            if (file != null) {
                val list = recentFiles.get().toMutableList()
                val path = file.absolutePath
                if (list.contains(path)) {
                    list.remove(path)
                }
                list.add(path)
                while (list.size > 15) {
                    list.removeFirst()
                }
                recentFiles.update(list)
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

        mainFileNavigator.subscribe(this, "Image navigation") { shift ->
            if (shift == 1) {
                nextImage()
            } else {
                prevImage()
            }
        }

        visibleChannel.subscribe(this, "Visible channel change") { channel ->
            updateCurrentShader(channel)
        }

        imageMonoColormap.subscribe(this, "Colormap change") { colormapName ->
            updateCurrentShader()
        }

        imageRGBColormap.subscribe(this, "Colormap change") { colormapName ->
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

    fun updateCurrentShader(channel: Int = visibleChannel.get()) {
        if (channel != 0 || mainImage.get()?.channels() == 1) {
            viewerShader.update(viewerShaderBuilder.getMono(imageMonoColormap.get()))
        } else {
            viewerShader.update(viewerShaderBuilder.getRGB(imageRGBColormap.get()))
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

    private fun nextImage() {
        val nextFile = fileManager.next(frameInterval.get())
        if (nextFile != null) {
            mainPath.update(nextFile.path)
        }
    }

    private fun prevImage() {
        val prevFile = fileManager.prev(frameInterval.get())
        if (prevFile != null) {
            mainPath.update(prevFile.path)
        }
    }

    fun loadDisplayProfile() {

    }

    fun saveDisplayProfile() {

    }

    fun loadFromClipboard() {
        ClipboardUtils.processClipboard({
            mainPath.update("clipboard")
        }, {
            val file = ClipboardUtils.getFileList()[0]
            mainPath.update(file.absolutePath)
        }, {
            mainPath.update(ClipboardUtils.getString())
        })
    }

    fun refreshMainPath() {
        mainPath.update(mainPath.get())
    }
}