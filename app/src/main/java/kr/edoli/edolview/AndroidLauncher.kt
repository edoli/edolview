package kr.edoli.edolview


import android.Manifest
import android.annotation.SuppressLint
import android.app.AlertDialog
import android.net.Uri
import android.os.Bundle
import android.system.Os
import android.util.Log
import android.util.TypedValue
import android.view.DragEvent
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.ScaleGestureDetector.OnScaleGestureListener
import androidx.core.app.ActivityCompat
import com.badlogic.gdx.Gdx
import com.badlogic.gdx.backends.android.AndroidApplication
import com.badlogic.gdx.backends.android.AndroidApplicationConfiguration
import com.badlogic.gdx.backends.android.AndroidGraphics
import com.badlogic.gdx.backends.android.AndroidInput
import com.badlogic.gdx.math.Vector2
import kr.edoli.edolview.ui.App
import kr.edoli.edolview.util.Platform
import kr.edoli.edolview.util.PlatformImp
import kr.edoli.edolview.util.TouchState
import org.apache.commons.io.IOUtils

class AndroidLauncher: AndroidApplication() {

	private val EXTERNAL_STORAGE_PERMISSION_CODE = 23

	private var touchState = TouchState.None

	@SuppressLint("ClickableViewAccessibility")
	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)

		Os.setenv("OPENCV_IO_ENABLE_OPENEXR", "1", true)
		Os.setenv("OPENCV_TEMP_PATH", cacheDir.absolutePath, true)

		ActivityCompat.requestPermissions(this, arrayOf(
			Manifest.permission.READ_EXTERNAL_STORAGE,
			Manifest.permission.WRITE_EXTERNAL_STORAGE),
			EXTERNAL_STORAGE_PERMISSION_CODE)

		Platform = object: PlatformImp {
			override fun showErrorMessage(message: String) {
				runOnUiThread {
					AlertDialog.Builder(context).setMessage(message).setTitle("Error").show()
				}
			}

			override fun getScalingFactor(): Float {
				return TypedValue.applyDimension(
					TypedValue.COMPLEX_UNIT_DIP,
					1.5f,
					resources.displayMetrics
				)
			}

			override fun contentResolve(uri: String): ByteArray? {
				val inURI = Uri.parse(uri)
				val stream = contentResolver.openInputStream(inURI)

				if (stream != null) {
					val bytes = IOUtils.toByteArray(stream)
					stream.close()
					return bytes
				}
				return null
			}

			override fun getTouchState() = touchState
		}

		val config = AndroidApplicationConfiguration()
		config.useGL30 = true

		val app = App("")
		initialize(app, config)

		val view = (Gdx.graphics as AndroidGraphics).view
		view.setOnDragListener { v, event ->

			when (event.action) {
				DragEvent.ACTION_DRAG_STARTED  -> {

					// Determines if this View can accept the dragged data.
//					if (e.clipDescription.hasMimeType(ClipDescription.MIMETYPE_TEXT_PLAIN)) {
//						(v as? ImageView)?.setColorFilter(Color.BLUE)
//						v.invalidate()
//						true
//					} else {
//						false
//					}
					true
				}
				DragEvent.ACTION_DRAG_ENTERED -> {
					true
				}
				DragEvent.ACTION_DRAG_LOCATION -> {
					true
				}
				DragEvent.ACTION_DRAG_EXITED -> {
					true
				}
				DragEvent.ACTION_DROP -> {
					requestDragAndDropPermissions(event)
					val item = event.clipData.getItemAt(0)
					val uri = item.uri.toString()

					Gdx.app.postRunnable {
						ImContext.mainPath.update(uri)
						Gdx.graphics.requestRendering()
					}
					true
				}

				DragEvent.ACTION_DRAG_ENDED -> {
					true
				}
				else -> {
					Log.e("EdolView", "Unknown action type received by View.OnDragListener.")
					false
				}
			}
		}

		val scaleGestureDetector = ScaleGestureDetector(context, object : OnScaleGestureListener {
			override fun onScale(detector: ScaleGestureDetector): Boolean {
				val newScale = detector.currentSpan / detector.previousSpan * ImContext.zoom.get()

				ImContext.zoomCenter.update(
					Vector2(detector.focusX, detector.focusY)
				)
				ImContext.zoom.update(newScale)
				return true
			}

			override fun onScaleBegin(detector: ScaleGestureDetector): Boolean {
				return true
			}

			override fun onScaleEnd(detector: ScaleGestureDetector) {
			}
		})

		view.setOnTouchListener { v, event ->
			if ((event.getToolType(0) and MotionEvent.BUTTON_STYLUS_PRIMARY) != 0) {
				touchState = TouchState.Secondary
			}

			event.action = when (event.action) {
				211 -> MotionEvent.ACTION_DOWN
				212 -> MotionEvent.ACTION_UP
				213 -> MotionEvent.ACTION_MOVE
				else -> event.action
			}

			scaleGestureDetector.onTouchEvent(event)
			if (!scaleGestureDetector.isInProgress) {
				return@setOnTouchListener (Gdx.input as AndroidInput).onTouch(v, event)
			} else {
				event.action = MotionEvent.ACTION_CANCEL
				(Gdx.input as AndroidInput).onTouch(v, event)
			}
			Gdx.graphics.requestRendering()

			touchState = TouchState.None
			return@setOnTouchListener true
		}

	}
}