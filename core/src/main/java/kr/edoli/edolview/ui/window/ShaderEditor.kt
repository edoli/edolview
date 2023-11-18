package kr.edoli.edolview.ui.window

import com.badlogic.gdx.Gdx
import kr.edoli.edolview.ImContext
import java.awt.Dimension
import java.awt.Font
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import java.awt.event.MouseAdapter
import java.awt.event.MouseEvent
import javax.swing.*
import javax.swing.border.EmptyBorder


class ShaderEditor private constructor(): BaseWindow() {
    companion object {
        var shaderEditor: ShaderEditor? = null

        fun show() {
            val editor = shaderEditor
            if (editor == null) {
                shaderEditor = ShaderEditor()
            } else {
                editor.isVisible = true
            }
        }
    }

    val shaderBuilder = ImContext.viewerShaderBuilder
    val prefShaders = ImContext.prefShaders
    val listSavedShaders = JList<String>()

    init {
        val container = contentPane
        container.layout = GridBagLayout()

        // Left panel
        val customShaderTextField = JTextArea(shaderBuilder.customShader)

        val logLabel = JTextArea()
        logLabel.isEditable = false

        val updateShader = {
            val lastCustomSahder = shaderBuilder.customShader
            shaderBuilder.customShader = customShaderTextField.text

            Gdx.app.postRunnable {
                try {
                    shaderBuilder.clearCache()
                    ImContext.updateCurrentShader()
                    logLabel.text = ""
                } catch (ex: Exception) {
                    logLabel.text = ex.stackTraceToString()
                    shaderBuilder.customShader = lastCustomSahder
                }
            }
        }

        container.add(JLabel("Input: vec4 tex, Output: vec4 gl_FragColor").apply {
            border = EmptyBorder(8, 8, 8, 8)
            font = font.deriveFont(font.style or Font.BOLD)
        }, GridBagConstraints().apply {
            grid(0, 0)
            fill = GridBagConstraints.HORIZONTAL
            weight(1.0, 0.0)
        })

        container.add(customShaderTextField, GridBagConstraints().apply {
            grid(0, 1)
            fill = GridBagConstraints.BOTH
            weighty = 1.0
        })

        customShaderTextField.addKeyListener(object : KeyListener {
            override fun keyTyped(e: KeyEvent?) {
            }

            override fun keyPressed(e: KeyEvent?) {
            }

            override fun keyReleased(e: KeyEvent) {
                if (e.keyCode == KeyEvent.VK_ENTER) {
                    updateShader()
                }
            }
        })

        container.add(JButton("Update shader").apply {
            addActionListener {
                updateShader()
            }
            alignmentX = CENTER_ALIGNMENT
        }, GridBagConstraints().apply {
            grid(0, 2)
            fill = GridBagConstraints.HORIZONTAL
        })

        container.add(logLabel, GridBagConstraints().apply {
            grid(0, 3)
            fill = GridBagConstraints.HORIZONTAL
        })

        // Right panel
        val nameTextField = JTextField()

        listSavedShaders.apply {
            selectionMode = ListSelectionModel.SINGLE_SELECTION

            addListSelectionListener {
                customShaderTextField.text = prefShaders.getString(this.selectedValue)
                nameTextField.text = this.selectedValue
            }

            addMouseListener(object : MouseAdapter() {
                override fun mousePressed(e: MouseEvent) {
                    if ( SwingUtilities.isRightMouseButton(e) ) {
                        val row = this@apply.locationToIndex(e.point)

                        val menu = JPopupMenu()
                        val itemRemove = JMenuItem("Remove")
                        itemRemove.addActionListener {
                            prefShaders.remove(this@apply.model.getElementAt(row))
                            prefShaders.flush()
                            updateShadersModel()
                        }
                        menu.add(itemRemove)

                        menu.show(this@apply, e.point.x, e.point.y)
                    }
                }
            })
        }

        updateShadersModel()

        val scollPane = object : JScrollPane(listSavedShaders) {
            override fun getPreferredSize(): Dimension {
                return Dimension(120, 0)
            }

            override fun getMinimumSize(): Dimension {
                return Dimension(120, 0)
            }
        }

        container.add(scollPane, GridBagConstraints().apply {
            grid(1, 0)
            gridheight = 2
            weight(0.0, 1.0)
            fill = GridBagConstraints.BOTH
        })

        container.add(nameTextField, GridBagConstraints().apply {
            grid(1, 2)
            fill = GridBagConstraints.HORIZONTAL
        })

        container.add(JButton("Save shader").apply {
            addActionListener {
                prefShaders.putString(nameTextField.text, customShaderTextField.text)
                prefShaders.flush()
                updateShadersModel()
            }
            alignmentX = CENTER_ALIGNMENT
        },
            GridBagConstraints().apply {
            grid(1, 3)
            fill = GridBagConstraints.HORIZONTAL
        })

        pack()
        title = "Shader editor"
        size = Dimension(480, 240)
        isVisible = true
    }

    private fun updateShadersModel() {
        val model = DefaultListModel<String>()

        val shadersDict = prefShaders.get()
        for (key in shadersDict.keys) {
            model.addElement(key)
        }

        listSavedShaders.model = model
    }
}