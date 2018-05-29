package kr.edoli.imview.ui

import javafx.scene.control.ContextMenu
import kr.edoli.imview.ImContext
import kr.edoli.imview.image.SelectBoxUtils
import kr.edoli.imview.util.ClipboardUtils
import tornadofx.action
import tornadofx.item

class ImContextMenu : ContextMenu() {
    init {
        item("Copy Image") {
            action {
                SelectBoxUtils.copyImageToClipboard()
            }
        }

        item("Copy Mean Color") {
            action {
                val color = SelectBoxUtils.selectBoxMeanColor()
                ClipboardUtils.putString(color.map { it.toInt() }.joinToString(","))
            }
        }

        item("Copy Rectangle Bound") {
            action {
                val rect = ImContext.selectBox.get()
                ClipboardUtils.putString(intArrayOf(rect.x, rect.y, rect.width, rect.height).joinToString(","))
            }
        }
    }
}