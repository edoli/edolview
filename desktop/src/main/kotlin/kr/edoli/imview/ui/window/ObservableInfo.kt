package kr.edoli.imview.ui.window

import kr.edoli.imview.util.ObservableContext
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import javax.swing.JFrame
import javax.swing.JScrollPane
import javax.swing.JTable

class ObservableInfo : JFrame() {
    init {
        title = "Good"

        val data = createData()
        val table = JTable(data, arrayOf("Name", "Count", "Update Time", "Subjects"))
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

    fun createData(): Array<Array<Any>> {
        val data = Array(ObservableContext.observableValues.size) { Array<Any>(4) {} }
        ObservableContext.observableValues.forEachIndexed { index, value ->
            data[index][0] = value.name
            data[index][1] = value.subscribers.size
            data[index][2] = "${value.lastTotalUpdateTime.toFloat() / 1000 / 1000}ms"
            data[index][3] = value.subscribers.map { it.subject }
        }
        return data
    }
}