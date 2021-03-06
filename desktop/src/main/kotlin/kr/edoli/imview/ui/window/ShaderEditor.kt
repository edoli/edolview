package kr.edoli.imview.ui.window

import com.badlogic.gdx.Gdx
import kr.edoli.imview.ImContext
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import javax.swing.JButton
import javax.swing.JFrame
import javax.swing.JTextArea
import javax.swing.JTextField

class ShaderEditor private constructor(): JFrame() {
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

    init {
        val container = contentPane
        val extraCodeTextField = JTextField(shaderBuilder.extraCode)
        val pixelExpressionTextField = JTextField(shaderBuilder.pixelExpression)
        val logLabel = JTextArea()
        logLabel.isEditable = false

        val updateShader = {
            val lastPixelExpression = shaderBuilder.pixelExpression
            shaderBuilder.extraCode = extraCodeTextField.text
            shaderBuilder.pixelExpression = pixelExpressionTextField.text

            Gdx.app.postRunnable {
                try {
                    shaderBuilder.clearCache()
                    ImContext.updateCurrentShader()
                    logLabel.text = ""
                } catch (ex: Exception) {
                    logLabel.text = ex.stackTraceToString()
                    shaderBuilder.pixelExpression = lastPixelExpression
                }
            }
        }

        val c = GridBagConstraints()
        c.weightx = 1.0

        container.layout = GridBagLayout()

        c.gridx = 0
        c.gridy = 0
        c.fill = GridBagConstraints.BOTH
        c.weighty = 1.0

        container.add(extraCodeTextField, c)
        extraCodeTextField.addKeyListener(object : KeyListener {
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

        c.gridx = 0
        c.gridy = 1
        c.fill = GridBagConstraints.HORIZONTAL
        c.weighty = 0.0
        container.add(pixelExpressionTextField, c)
        pixelExpressionTextField.addKeyListener(object : KeyListener {
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

        c.gridy = 2
        c.fill = GridBagConstraints.HORIZONTAL
        c.weighty = 0.0
        container.add(JButton("Update shader").apply {
            addActionListener {
                updateShader()
            }
            alignmentX = CENTER_ALIGNMENT
        }, c)

        c.gridy = 3
        container.add(logLabel, c)

        pack()
        isVisible = true
    }
}