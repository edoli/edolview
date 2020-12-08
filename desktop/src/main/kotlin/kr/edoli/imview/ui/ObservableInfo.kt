package kr.edoli.imview.ui

import kr.edoli.imview.util.ObservableContext
import javax.swing.JFrame
import javax.swing.JTable

class ObservableInfo : JFrame() {
    init {
        title = "Good"
        val data = Array(ObservableContext.observableValues.size) { Array<Any>(2) {} }
        ObservableContext.observableValues.forEachIndexed { index, value ->
            data[index][0] = value.name
            data[index][1] = value.subjects.size
        }
        val table = JTable(data, arrayOf("1", "1"))
        add(table)
        pack()
        isVisible = true
    }
}