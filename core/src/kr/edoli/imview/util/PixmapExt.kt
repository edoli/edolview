package kr.edoli.imview.util

import com.badlogic.gdx.graphics.Pixmap

/**
 * Created by sjjeon on 16. 10. 18.
 */

fun Pixmap.getChannels(): Int {
    return if (format == Pixmap.Format.RGBA4444 || format == Pixmap.Format.RGBA8888) 4
        else if (format == Pixmap.Format.RGB565 || format == Pixmap.Format.RGB888) 3
        else 1
}