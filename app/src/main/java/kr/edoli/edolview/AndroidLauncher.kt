package kr.edoli.edolview

import android.app.AlertDialog
import android.os.Bundle
import android.util.TypedValue
import com.badlogic.gdx.backends.android.AndroidApplication
import com.badlogic.gdx.backends.android.AndroidApplicationConfiguration
import kr.edoli.edolview.ui.App
import kr.edoli.edolview.util.Platform
import kr.edoli.edolview.util.PlatformImp

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
					2f,
					resources.displayMetrics
				)
			}
		}

		val config = AndroidApplicationConfiguration()
		initialize(App(""), config)
	}
}