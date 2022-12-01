package kr.edoli.edolview

import android.app.ActivityManager
import android.app.AlertDialog
import android.content.Context
import android.net.Uri
import android.os.Bundle
import android.util.Log
import android.util.TypedValue
import android.view.DragEvent
import android.view.MotionEvent
import android.view.ScaleGestureDetector
import android.view.ScaleGestureDetector.OnScaleGestureListener
import com.badlogic.gdx.Gdx
import com.badlogic.gdx.backends.android.AndroidApplication
import com.badlogic.gdx.backends.android.AndroidApplicationConfiguration
import com.badlogic.gdx.backends.android.AndroidGraphics
import com.badlogic.gdx.backends.android.AndroidInput
import com.badlogic.gdx.math.Vector2
import kr.edoli.edolview.ui.App
import kr.edoli.edolview.ui.MainScreen
import kr.edoli.edolview.util.Platform
import kr.edoli.edolview.util.PlatformImp
import org.apache.commons.io.IOUtils

class AndroidLauncher: AndroidApplication() {

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)

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
				val stream = contentResolver.openInputStream(Uri.parse(uri))

				if (stream != null) {
					val bytes = IOUtils.toByteArray(stream)
					stream.close()
					return bytes
				}
				return null
			}
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
					ImContext.mainPath.update(item.uri.toString())
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
			scaleGestureDetector.onTouchEvent(event)
			if (!scaleGestureDetector.isInProgress) {
				return@setOnTouchListener (Gdx.input as AndroidInput).onTouch(v, event)
			} else {
				event.action = MotionEvent.ACTION_CANCEL
				(Gdx.input as AndroidInput).onTouch(v, event)
			}
			Gdx.graphics.requestRendering()
			return@setOnTouchListener true
		}

	}
}