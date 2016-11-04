package kr.edoli.imview.ui.screen

import com.badlogic.gdx.scenes.scene2d.ui.ScrollPane
import kr.edoli.imview.res.Colors
import kr.edoli.imview.ui.ColorWidget
import kr.edoli.imview.ui.view.ImageListViewer

/**
 * Created by daniel on 16. 9. 24.
 */
class ImageListScreen : BaseScreen() {


    val imageListViewer = ImageListViewer()
    val scrollPane = ScrollPane(imageListViewer)

    init {
        val background = ColorWidget(Colors.background)
        background.setFillParent(true)

        scrollPane.setFillParent(true)
        imageListViewer.setFillParent(true)


        stage.addActor(background)
        stage.addActor(scrollPane)
    }

    override fun resize(width: Int, height: Int) {
        super.resize(width, height)

        scrollPane.invalidate()
        imageListViewer.invalidate()
    }
}