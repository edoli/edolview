package kr.edoli.imview.ui.window

import com.badlogic.gdx.Gdx
import kr.edoli.imview.ImContext
import java.awt.Dimension
import java.awt.GridBagConstraints
import java.awt.GridBagLayout
import java.awt.Rectangle
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import java.lang.Exception
import javax.swing.*

class ShaderEditor : JFrame() {
    val shaderBuilder = ImContext.viewerShaderBuilder

    init {
        val container = contentPane
        val extraCodeTextField = JTextField(shaderBuilder.extra_code)
        val pixelExpressionTextField = JTextField(shaderBuilder.pixel_expression)
        val logLabel = JTextArea()
        logLabel.isEditable = false

        val updateShader = {
            val lastPixelExpression = shaderBuilder.pixel_expression
            shaderBuilder.extra_code = extraCodeTextField.text
            shaderBuilder.pixel_expression = pixelExpressionTextField.text

            Gdx.app.postRunnable {
                try {
                    val shader = shaderBuilder.build()
                    ImContext.viewerShader.update(shader)
                    logLabel.text = ""
                } catch (ex: Exception) {
                    logLabel.text = ex.stackTraceToString()
                    shaderBuilder.pixel_expression = lastPixelExpression
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