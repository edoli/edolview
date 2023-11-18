package kr.edoli.edolview.ui.window

import javax.swing.ImageIcon
import javax.swing.JFrame

abstract class BaseWindow : JFrame() {
    init {
        iconImage = ImageIcon("icon.png").image
    }
}