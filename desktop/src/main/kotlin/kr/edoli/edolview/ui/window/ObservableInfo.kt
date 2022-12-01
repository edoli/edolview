package kr.edoli.edolview.ui.window

import kr.edoli.edolview.util.ObservableContext
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import javax.swing.JFrame
import javax.swing.JScrollPane
import javax.swing.JTable
import kotlin.concurrent.fixedRateTimer

class ObservableInfo private constructor() : JFrame() {
    companion object {
        var infoPanel: ObservableInfo? = null

        fun show() {
            val panel = infoPanel
            if (panel == null) {
                infoPanel = ObservableInfo()
            } else {
                panel.isVisible = true
            }
        }
    }

    val refresher = fixedRateTimer(period = 1000) {
        if (isVisible) {
            val newData = createData()

            (0 until table.rowCount).forEach { i ->
                (0 until table.columnCount).forEach { j ->
                    table.setValueAt(newData[i][j], i, j)
                }
            }
        }
    }

    val table = JTable(createData(), arrayOf("Name", "Count", "Update Time", "Subjects"))

    init {
        title = "Info"

        (0 until table.rowCount).forEach { i ->
            (0 until table.columnCount).forEach { j ->
                table.editCellAt(i, j)
            }
        }
        table.autoResizeMode = JTable.AUTO_RESIZE_LAST_COLUMN
        table.columnModel.getColumn(1).preferredWidth = 32
        table.columnModel.getColumn(3).preferredWidth = 500

        add(JScrollPane(table))

        table.addKeyListener(object : KeyListener {
            override fun keyTyped(e: KeyEvent) {
            }

            override fun keyPressed(e: KeyEvent) {
                if (e.keyCode == KeyEvent.VK_F5) {
                    val newData = createData()
                    newData.forEachIndexed { i, row ->
                        row.forEachIndexed { j, value ->
                            table.model.setValueAt(value, i, j)
                        }
                    }
                }
            }

            override fun keyReleased(e: KeyEvent) {
            }

        })
        pack()

        table.isFocusable = true
        isVisible = true

    }

    override fun dispose() {
        super.dispose()
        refresher.cancel()
    }

    fun createData(): Array<Array<Any>> {
        val allSize = ObservableContext.observables.size +
                ObservableContext.observableValues.size +
                ObservableContext.observableLazyValues.size +
                ObservableContext.observableLists.size

        val data = Array(allSize) { Array<Any>(4) {} }
        var index = 0
        ObservableContext.observables.forEachIndexed { _, value ->
            data[index][0] = value.name
            data[index][1] = value.subscribers.size
            data[index][2] = "${value.lastTotalUpdateTime.toFloat() / 1000 / 1000}ms"
            data[index][3] = value.subscribers.map { it.subject }
            index += 1
        }
        ObservableContext.observableValues.forEachIndexed { _, value ->
            data[index][0] = value.name
            data[index][1] = value.subscribers.size
            data[index][2] = "${value.lastTotalUpdateTime.toFloat() / 1000 / 1000}ms"
            data[index][3] = value.subscribers.map { it.subject }
            index += 1
        }
        ObservableContext.observableLazyValues.forEachIndexed { _, value ->
            data[index][0] = value.name
            data[index][1] = value.subscribers.size
            data[index][2] = "${value.lastTotalUpdateTime.toFloat() / 1000 / 1000}ms"
            data[index][3] = value.subscribers.map { it.subject }
            index += 1
        }
        ObservableContext.observableLists.forEachIndexed { i, value ->
            data[index][0] = value.name
            data[index][1] = value.subscribers.size
            data[index][2] = "${value.lastTotalUpdateTime.toFloat() / 1000 / 1000}ms"
            data[index][3] = value.subscribers.map { it.subject }
            index += 1
        }
        return data
    }
}