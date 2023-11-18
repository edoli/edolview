package kr.edoli.edolview.ui.window

import java.awt.GridBagConstraints

fun GridBagConstraints.grid(x: Int, y: Int) {
    gridx = x
    gridy = y
}

fun GridBagConstraints.weight(x: Double, y: Double) {
    weightx = x
    weighty = y
}

fun GridBagConstraints.size(width: Int, height: Int) {
    gridwidth = width
    gridheight = height
}