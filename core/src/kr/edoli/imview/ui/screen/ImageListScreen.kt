package kr.edoli.imview.ui.screen

import com.badlogic.gdx.scenes.scene2d.ui.ScrollPane
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.ColorWidget
import kr.edoli.imview.ui.view.ImageListViewer

/**
 * Created by daniel on 16. 9. 24.
 */
class ImageListScreen : BaseScreen() {

    init {
        val background = ColorWidget(Colors.background)
        background.setFillParent(true)

        val imageListViewer = ImageListViewer()
        val scrollPane = ScrollPane(imageListViewer)

        scrollPane.setFillParent(true)


        stage.addActor(background)
        stage.addActor(scrollPane)
    }
}